mod app;
mod config;
mod elo;
mod game;
mod player;
mod state;

use app::App;
use state::State;

use std::error::Error;

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
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap();

    let mut app = App::new(config, state, client);
    app.run().await
}
