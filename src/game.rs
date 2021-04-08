use reqwest::header::{HeaderMap, HeaderValue, REFERER};
use serde::Deserialize;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::sleep;

use crate::config::Config;
use crate::elo::Winner;

/// The different events a match can emit.
/// N.B. this does not take into account team fights.
#[derive(Debug, PartialEq)]
pub enum Event {
    Unknown,
    Opened(String, String),
    Locked,
    Decided(Winner, String, String),
}

/// States of a match. Not all fields are used, some are specified solely to
/// make serde/reqwest happy when deserializing the response from the server.S
///
/// Side-note: this format of the response hasn't changed in ~3 years.
/// N.B. this does not take into account team fights.
#[derive(Deserialize, Default, PartialEq, Clone)]
struct State {
    p1name: String,
    p2name: String,
    p1total: String,
    p2total: String,
    status: String,
    alert: String,
    x: u8,
    remaining: String,
}

pub struct Game {}

impl Game {
    pub async fn login(
        client: &mut reqwest::Client,
        config: &Config,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut headers = HeaderMap::new();
        headers.insert(
            REFERER,
            HeaderValue::from_str(&config.url_referer.as_str()).unwrap(),
        );

        let params = [
            ("email", config.username.as_str()),
            ("pword", config.password.as_str()),
            ("authenticate", "signin"),
        ];

        client
            .post(&config.url_login)
            .headers(headers)
            .form(&params)
            .send()
            .await?;

        Ok(())
    }

    pub async fn get_balance(
        client: &mut reqwest::Client,
        config: &Config,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }

    /// Sends an `Event` to the `outbox` specified. We only send an event when the
    /// current match state has changed.
    pub async fn stream(
        url: String,
        outbox: mpsc::Sender<Event>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut current_state: State = Default::default();
        loop {
            sleep(Duration::from_secs(5)).await;
            if let Ok(r) = reqwest::get(&url).await {
                if let Ok(state) = r.json::<State>().await {
                    if current_state == state {
                        continue;
                    }
                    Self::process_stream_event(state.clone(), &outbox).await?;
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
    async fn process_stream_event(
        state: State,
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
            outcome => {
                let winner = match outcome {
                    "1" => Winner::One,
                    "2" => Winner::Two,
                    _ => Winner::Draw,
                };
                outbox
                    .send(Event::Decided(winner, state.p1name, state.p2name))
                    .await?
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_process_stream_event_player_one_wins() {
        let mut state: State = Default::default();
        state.status = String::from("1");
        state.p1name = String::from("winner");
        state.p1name = String::from("loser");

        let (outbox, mut inbox) = mpsc::channel(1);
        let _ = Game::process_stream_event(state.clone(), &outbox).await;
        let response = inbox.recv().await;
        assert!(response.is_some());

        let message = response.expect("failed");
        assert_eq!(
            message,
            Event::Decided(Winner::One, state.p1name, state.p2name)
        );
    }
}
