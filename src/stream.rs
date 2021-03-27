use futures::StreamExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::io;
use tokio::sync::mpsc;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{Error, Result},
};

const NOTIFICATION_BUFFER_SIZE: usize = 1024;

#[derive(Debug)]
pub enum Event {
    Opened(String, String),
    Closed,
    Decided(String),
}

pub struct Stream {}

impl Stream {
    pub async fn start(server: String, sink: mpsc::Sender<Event>) -> Result<()> {
        let (stream, _) = connect_async(server).await?;
        let (tx, rx) = stream.split();

        rx.for_each(|message| async {
            if let Ok(data) = message {
                println!("{:?}", data);
            }
        }).await;

        Ok(())
    }
}
