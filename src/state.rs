use rusqlite::{named_params, params, Connection};
use std::error::Error;
use std::fmt;
use std::path::Path;
use std::result;

use crate::config::Config;
use crate::elo::{Elo, Winner};
use crate::game::Event;
use crate::player::Player;

/// This struct is really just a wrapper for some functions which manage storing
/// our state.  In reality, they mostly take a `&rusqlite::Connection` as their
/// first argument.
pub struct State {}

/// I apologize for the long word.
#[derive(Debug, Default)]
struct NoValidDatabaseError {}
impl Error for NoValidDatabaseError {}
impl fmt::Display for NoValidDatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Valid options are: 'memory' and a path to a file.")
    }
}

/// I apologize for the long word.
#[derive(Debug, Default)]
struct CouldNotCreateDatabaseConnectionError {}
impl Error for CouldNotCreateDatabaseConnectionError {}
impl fmt::Display for CouldNotCreateDatabaseConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Couldn't create a database connection.")
    }
}

impl State {
    /// We only know about two kinds of "database storage" methods:
    /// - memory: don't persist anything;
    /// - on disk: persist everything to an SQLite3 database.
    pub fn new(config: &Config) -> result::Result<Connection, Box<dyn Error>> {
        let state = match config.file_db.as_str() {
            "memory" => Connection::open_in_memory()?,
            path => Connection::open(&Path::new(path))?,
        };

        if let Err(_) = state.execute_batch(include_str!("schema.sql")) {
            Err(Box::new(CouldNotCreateDatabaseConnectionError {}))
        } else {
            Ok(state)
        }
    }

    /// Get the specified player or return a default `Player`.
    pub fn get_player(state: &Connection, name: &String) -> Player {
        let player = state
            .prepare("SELECT name, elo FROM players WHERE name = :name;")
            .and_then(|mut stmt| {
                stmt.query_row_named(named_params! { ":name": name}, |row| {
                    Ok(Player::new(row.get(0)?, Elo::with_rating(row.get(1)?)))
                })
            });

        match player {
            Ok(p) => p,
            _ => Player::new((&name).to_string(), Elo::new()),
        }
    }

    /// Upsert information about a `Player`.
    pub fn put_player(state: &Connection, player: &Player) {
        let _ = state.prepare(
            "INSERT INTO players (name, elo) VALUES (:name, :elo) ON CONFLICT (name) DO UPDATE SET elo = :elo;"
        ).and_then(|mut stmt| {
            stmt.execute_named(named_params! {
                ":name": player.name,
                ":elo": player.elo.rating,
            })
        });
    }

    /// Add fight information.
    pub fn put_event(state: &Connection, event: &Event) {
        if let Event::Decided(winner, player_one, player_two) = event {
            let fight = state.prepare(
                "
                WITH
                    player_one(id) AS (SELECT id FROM players WHERE name = :one),
                    player_two(id) AS (SELECT id FROM players WHERE name = :two)
                INSERT INTO fights (ended, winner, one, two)
                    SELECT datetime('now'), :winner, player_one.id, player_two.id FROM player_one, player_two;
                "
            ).and_then(|mut stmt| {
                stmt.execute_named(named_params! {
                    ":winner": winner,
                    ":one": player_one,
                    ":two": player_two,
                })
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_put_get_player() -> result::Result<(), Box<dyn Error>> {
        let mut config: Config = Default::default();
        config.file_db = String::from("memory");

        let state = State::new(&config)?;
        let player_orig = Player::new(String::from("test"), Elo::with_rating(1337));

        State::put_player(&state, &player_orig);
        let player_saved = State::get_player(&state, &String::from("test"));

        assert_eq!(player_orig.elo.rating, player_saved.elo.rating);
        Ok(())
    }

    #[test]
    fn test_put_event_fail_no_player() -> result::Result<(), Box<dyn Error>> {
        let mut config: Config = Default::default();
        config.file_db = String::from("memory");

        let state = State::new(&config)?;
        let event = Event::Decided(Winner::One, String::from("one"), String::from("two"));
        State::put_event(&state, &event);

        Ok(())
    }
}
