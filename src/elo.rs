use rusqlite;
use rusqlite::types::ToSqlOutput;
use std::convert::From;
use std::fmt;

const ALGORITHM_OF: f32 = 400f32;
const K_FACTOR: f32 = 32f32;

#[derive(Debug, Clone, Copy)]
pub struct Elo {
    pub rating: i32,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Winner {
    Draw,
    One,
    Two,
}

impl rusqlite::ToSql for Winner {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(match self {
            Winner::One => 1,
            Winner::Two => 2,
            Winner::Draw => 3,
        }))
    }
}

impl From<&Winner> for String {
    fn from(item: &Winner) -> Self {
        match item {
            Winner::One => String::from("player1"),
            Winner::Two => String::from("player2"),
            Winner::Draw => String::from(""),
        }
    }
}

impl From<String> for Winner {
    fn from(item: String) -> Self {
        match item.as_str() {
            "player1" => Winner::One,
            "player2" => Winner::Two,
            _ => Winner::Draw,
        }
    }
}

impl From<Winner> for u32 {
    fn from(item: Winner) -> Self {
        match item {
            Winner::One => 1,
            Winner::Two => 2,
            Winner::Draw => 3,
        }
    }
}

impl From<u32> for Winner {
    fn from(item: u32) -> Self {
        match item {
            1 => Winner::One,
            2 => Winner::Two,
            _ => Winner::Draw,
        }
    }
}

impl fmt::Display for Winner {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Winner::Draw => write!(f, "Draw"),
            Winner::One => write!(f, "One"),
            Winner::Two => write!(f, "Two"),
        }
    }
}

impl Elo {
    /// Create a new `Elo` struct with a rating of 1000.
    pub fn new() -> Self {
        Elo { rating: 1000 }
    }

    /// Create a new `Elo` struct with the given rating.
    pub fn with_rating(rating: i32) -> Self {
        Self { rating }
    }

    /// Calculate the expected outcome of a challenge against
    /// the `opponent`.
    ///
    /// # Example
    ///
    /// ```
    /// player = Elo::new()
    /// player.expected(Elo::new())
    /// ```
    pub fn expected(one: &Elo, two: &Elo) -> f32 {
        // Expected = Chance of winning + (Chance of drawing / 2)
        //
        // A value of 0.5 means either:
        //
        // 1.  0% chance of win, 100% chance of draw,  0% chance of lose.
        // 2. 25% chance of win,  50% chance of draw, 25% chance of lose;
        // 3. 50% chance of win,   0% chance of draw, 50% chance of lose;
        //
        // When the players' ratings are the same we'll have to return this value because otherwise
        // we'll make the Math Gods angry.

        if two.rating == one.rating {
            0.5
        } else {
            let rating: f32 = (two.rating - one.rating) as f32 / ALGORITHM_OF;
            1f32 / (1f32 + 10f32.powf(rating))
        }
    }

    /// Update the rankings for the winner and loser.
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut white = Elo::with_rating(800);
    /// let white_orig = white.clone();
    /// let mut black = Elo::new();
    /// Elo::update_ratings(&mut black, &mut white);
    /// ```
    pub fn update_ratings(winner: Winner, one: &mut Elo, two: &mut Elo) {
        // The expected outcome for the two will be `1 - expected outcome for one`. This
        // allows us to simplify some math.
        let expected = Self::expected(&one, &two);

        match winner {
            Winner::One => {
                one.rating = (one.rating as f32 + K_FACTOR * (1f32 - expected)).floor() as i32;
                two.rating = (two.rating as f32 + K_FACTOR * (-1f32 + expected)).floor() as i32;
            }
            Winner::Two => {
                one.rating = (one.rating as f32 + K_FACTOR * (-expected)).floor() as i32;
                two.rating = (two.rating as f32 + K_FACTOR * expected).floor() as i32;
            }
            Winner::Draw => {
                two.rating = (two.rating as f32 + K_FACTOR * 0.5f32).floor() as i32;
                one.rating = (one.rating as f32 + K_FACTOR * 0.5f32).floor() as i32;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elo_expect_draw() {
        let one = Elo::new();
        let two = Elo::new();
        assert_eq!(Elo::expected(&one, &two), 0.5f32);
    }

    #[test]
    fn test_elo_expect_one_should_lose() {
        let one = Elo::with_rating(800);
        let two = Elo::new();
        assert!(Elo::expected(&one, &two) < 0.5f32);
    }

    #[test]
    fn test_elo_update_one_lose() {
        let mut one = Elo::with_rating(800);
        let one_orig = one.clone();
        let mut two = Elo::new();
        Elo::update_ratings(Winner::Two, &mut one, &mut two);
        Elo::update_ratings(Winner::Two, &mut one, &mut two);
        Elo::update_ratings(Winner::Two, &mut one, &mut two);
        Elo::update_ratings(Winner::Two, &mut one, &mut two);
        Elo::update_ratings(Winner::Two, &mut one, &mut two);
        assert!(one.rating < one_orig.rating);
    }

    #[test]
    fn test_elo_update_unbalanced_draw_one_lose() {
        let mut one = Elo::with_rating(800);
        let mut two = Elo::new();
        Elo::update_ratings(Winner::Draw, &mut one, &mut two);
        Elo::update_ratings(Winner::Draw, &mut one, &mut two);
        Elo::update_ratings(Winner::Draw, &mut one, &mut two);
        assert!(two.rating > one.rating);
    }

    #[test]
    fn test_elo_update_balanced_draw() {
        let mut one = Elo::new();
        let mut two = Elo::new();
        Elo::update_ratings(Winner::Draw, &mut one, &mut two);
        Elo::update_ratings(Winner::Draw, &mut one, &mut two);
        Elo::update_ratings(Winner::Draw, &mut one, &mut two);
        assert_eq!(two.rating, one.rating);
    }
}
