mod elo;
mod player;
mod state;
mod stream;

use tokio::sync::mpsc;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (outbox, inbox) = mpsc::channel(1);
    stream::Stream::start(String::from("ws://localhost:1337"), outbox).await?;
    Ok(())
}
