mod elo;
mod player;
mod state;
mod stream;

use std::error::Error;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (outbox, mut inbox) = mpsc::channel(1);
    let url = String::from("https://www.saltybet.com/state.json");

    let job = tokio::spawn(stream::EventStream::start(url, outbox));

    while let Some(event) = inbox.recv().await {
        println!("stream: {:?}", event);
    }

    tokio::join!(job);

    Ok(())
}
