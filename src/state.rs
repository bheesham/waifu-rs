use rusqlite::{named_params, params, Connection, Result};

use crate::elo::Elo;
use crate::player::Player;

pub struct State {
    db: Connection,
}

impl State {
    pub fn new() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute(
            "
                CREATE TABLE IF NOT EXISTS players (
                    name    TEXT NOT NULL UNIQUE ON CONFLICT REPLACE,
                    rating  INTEGER NOT NULL DEFAULT(1000)
                );
            ",
            params![],
        )?;

        Ok(Self { db: conn })
    }

    pub fn get_player(&mut self, name: String) -> Player {
        let player = self
            .db
            .prepare("SELECT name, rating FROM players WHERE name = :name;")
            .and_then(|mut stmt| {
                stmt.query_row_named(named_params! { ":name": name}, |row| {
                    Ok(Player::new(row.get(0)?, Elo::with_rating(row.get(1)?)))
                })
            });

        match player {
            Ok(p) => p,
            _ => Player::new(name, Elo::new()),
        }
    }

    pub fn put_player(&mut self, player: &Player) {
        let _ = self.db.prepare(
            "INSERT INTO players (name, rating) VALUES (:name, :rating) ON CONFLICT (name) DO UPDATE SET rating=:rating;"
        ).and_then(|mut stmt| {
            stmt.execute_named(named_params! {
                ":name": player.name,
                ":rating": player.rating.rating,
            })
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_put_get_player() -> Result<()> {
        let mut state = State::new()?;
        let player_orig = Player::new(String::from("test"), Elo::with_rating(1337));

        state.put_player(&player_orig);
        let player_saved = state.get_player(String::from("test"));

        assert_eq!(player_orig.rating.rating, player_saved.rating.rating);
        Ok(())
    }
}
