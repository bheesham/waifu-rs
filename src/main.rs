mod config;
mod elo;
mod game;
mod player;
mod state;

use elo::{Elo, Winner};
use state::State;

use log::{error, info, trace};
use std::error::Error;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    // Read from the environment. Maybe swap this out later.
    let config = config::configure();

    // We'll need a place to store our state. I made a teeny wrapper to hide
    // some of the implementation deets.
    let state = State::new(&config).expect("Could not create a database connection.");

    // This client needs to be mutable since we start off with a "clean" client,
    // authenticate (by logging in), and modify it so it stores cookies. This
    // way, when we go to make a subsequent request to place a bet we don't need
    // to worry (too much) about not being logged in.
    let mut client = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap();

    // We use the client we created above in a mutable way: send off a login request
    // to the SB website.
    if let Err(e) = game::Game::login(&mut client, &config).await {
        panic!("Could not log in! Error: {}", e);
    }

    // Start a shared channel so we can get events from the stream.
    let (outbox, mut inbox) = mpsc::channel(1);

    // Start the stream.
    let stream = tokio::spawn(game::Game::stream(config.url_state.clone(), outbox));
    while let Some(event) = inbox.recv().await {
        match event {
            game::Event::Opened(ref one_name, ref two_name) => {
                let one = State::get_player(&state, &one_name);
                let two = State::get_player(&state, &two_name);
                let expected_winner = {
                    if Elo::expected(&one.elo, &two.elo) >= 0.5f32 {
                        Winner::One
                    } else {
                        Winner::Two
                    }
                };

                // The logic here is:
                // 1. try placing a bet;
                // 2. if no bet could be placed, login again;
                // 3. place bet again;
                // 4. if no bet could be placed, bail!
                if let Err(_) = game::Game::place_bet(&mut client, &expected_winner, &config).await
                {
                    if let Ok(_) = game::Game::login(&mut client, &config).await {
                        let _ =
                            game::Game::place_bet(&mut client, &expected_winner, &config).await?;
                    } else {
                        panic!(
                            "Cookies and credentials expired. Gotta bail to not wreak havoc on SaltyBet."
                        );
                    }
                }

                info!(
                    "Placed a bet on: {}",
                    match expected_winner {
                        Winner::One => one.name.as_str(),
                        Winner::Two => two.name.as_str(),
                        _ => "Unknown?",
                    }
                );
            }
            game::Event::Decided(ref winner, ref one_name, ref two_name) => {
                let mut one = State::get_player(&state, &one_name);
                let mut two = State::get_player(&state, &two_name);
                Elo::update_ratings(*winner, &mut one.elo, &mut two.elo);
                State::put_player(&state, &one);
                State::put_player(&state, &two);
                State::put_event(&state, &event);
                info!("winner: {}; one: {}; two: {}", winner, one.name, two.name);
            }
            _ => {}
        }
    }

    let _ = tokio::join!(stream);
    Ok(())
}
