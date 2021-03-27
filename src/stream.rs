use std::time::Duration;
use hyper::Client;
use tokio::time::sleep;
use tokio::io;
use tokio::sync::mpsc;

const NOTIFICATION_BUFFER_SIZE: usize = 1024;

#[derive(Debug)]
pub enum Event {
    Opened(String, String),
    Closed,
    Decided(String),
}

pub struct Stream {}

impl Stream {
    pub async fn start(url: String, sink: mpsc::Sender<Event>) -> Result<(), Box<dyn std::error::Error + Send + Sync>>{
        let client = Client::new();

        loop {
            let resp = client.get(url.parse()?).await;

            println!("{:?}", resp);

            sink.send(Event::Closed).await?;
            sleep(Duration::from_secs(10)).await;
        }
    }
}
