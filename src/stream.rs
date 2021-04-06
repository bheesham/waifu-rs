use serde::Deserialize;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::sleep;

/// The different events a match can emit.
/// N.B. this does not take into account team fights.
#[derive(Debug, PartialEq)]
pub enum Event {
    Unknown,
    Opened(String, String),
    Locked,
    Decided(String, String),
}

/// States of a match. Not all fields are used, some are specified solely to
/// make serde/reqwest happy when deserializing the response from the server.S
///
/// Side-note: this format of the response hasn't changed in ~3 years.
/// N.B. this does not take into account team fights.
#[derive(Deserialize, Default, PartialEq, Clone)]
struct MatchState {
    p1name: String,
    p2name: String,
    p1total: String,
    p2total: String,
    status: String,
    alert: String,
    x: u8,
    remaining: String,
}

pub struct EventStream {}

impl EventStream {
    /// Sends an `Event` to the `outbox` specified. We only send an event when the
    /// current match state has changed.
    pub async fn start(
        url: String,
        outbox: mpsc::Sender<Event>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut current_state: MatchState = Default::default();
        loop {
            sleep(Duration::from_secs(5)).await;
            if let Ok(r) = reqwest::get(url.clone()).await {
                if let Ok(state) = r.json::<MatchState>().await {
                    if current_state == state {
                        continue;
                    }
                    Self::process(state.clone(), &outbox).await?;
                    current_state = state;
                }
            }
        }
    }

    /// Figure out which event to send to the `outbox`.
    /// The values for this are:
    /// - locked: betting has been locked;
    /// - open: you can place your bets for either p1 or p2;
    /// - 1: player 1 has won;
    /// - 2: player 2 has won.
    ///
    /// N.B. this does not take into account team fights.
    async fn process(
        state: MatchState,
        outbox: &mpsc::Sender<Event>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match state.status.as_str() {
            "locked" => {
                outbox.send(Event::Locked).await?;
            }
            "open" => {
                outbox
                    .send(Event::Opened(state.p1name, state.p2name))
                    .await?;
            }
            "1" => {
                outbox
                    .send(Event::Decided(state.p1name, state.p2name))
                    .await?
            }
            "2" => {
                outbox
                    .send(Event::Decided(state.p2name, state.p1name))
                    .await?
            }
            _ => outbox.send(Event::Unknown).await?,
        }

        Ok(())
    }
}
