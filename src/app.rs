use crate::config::Config;
use crate::elo::{Elo, Winner};
use crate::state::State;

use crate::game;

use rusqlite::Connection;
use tokio::sync::mpsc;
use log::{info, error};
use std::error::Error;

pub struct App {
    config: Config,
    db: Connection,
    http_client: reqwest::Client,
}

impl App {
    pub fn new(config: Config, db: Connection, http_client: reqwest::Client) -> Self {
        Self {
            config,
            db,
            http_client,
        }
    }

    /// Wrapper around an infinite loop where we will catch an error, log it, then retry.
    #[allow(unreachable_code)]
    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            if let Err(e) = self.step().await {
                error!("Failed in the game loop: {}", e);
            }
        }
        Ok(())
    }

    /// Performs the main loop of the program, which in ordinary circumstances should not exit.
    /// Every once in a while though, it will, so we isolate that functionality and retry at a
    /// higher level.
    async fn step(&mut self) -> Result<(), Box<dyn Error>> {
        // We use the client we created above in a mutable way: send off a login request
        // to the SB website.
        if let Err(e) = game::Game::login(&mut self.http_client, &self.config).await {
            panic!("Could not log in! Error: {}", e);
        }

        // Start a shared channel so we can get events from the stream.
        let (outbox, mut inbox) = mpsc::channel(1);

        // Start the stream.
        let stream = tokio::spawn(game::Game::stream(self.config.url_state.clone(), outbox));
        while let Some(event) = inbox.recv().await {
            match event {
                game::Event::Opened(ref one_name, ref two_name) => {
                    let one = State::get_player(&self.db, &one_name);
                    let two = State::get_player(&self.db, &two_name);
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
                    if let Err(_) = game::Game::place_bet(&mut self.http_client, &expected_winner, &self.config).await
                    {
                        if let Ok(_) = game::Game::login(&mut self.http_client, &self.config).await {
                            let _ =
                                game::Game::place_bet(&mut self.http_client, &expected_winner, &self.config).await?;
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
                    let mut one = State::get_player(&self.db, &one_name);
                    let mut two = State::get_player(&self.db, &two_name);
                    Elo::update_ratings(*winner, &mut one.elo, &mut two.elo);
                    State::put_player(&self.db, &one);
                    State::put_player(&self.db, &two);
                    State::put_event(&self.db, &event);
                    info!("winner: {}; one: {}; two: {}", winner, one.name, two.name);
                }
                _ => {}
            }
        }

        let _ = tokio::join!(stream);
        Ok(())
    }
}
