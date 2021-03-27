mod elo;
mod player;
mod state;
mod stream;

use std::time::Duration;
use hyper::Client;
use tokio::time::sleep;
use tokio::sync::mpsc;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (outbox, mut inbox) = mpsc::channel(10);
    let url = String::from("https://www.saltybet.com/state.json").parse()?;

    tokio::spawn(async move {
        stream::Stream::start(url, outbox).await;
    });

    while let Some(event) = inbox.recv().await {
        println!("{:?}", event);
    }
    Ok(())
}
