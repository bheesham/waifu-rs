const ALGORITHM_OF: f32 = 400f32;
const K_FACTOR: f32 = 32f32;

#[derive(Debug, Clone, Copy)]
pub struct Elo {
    pub rating: i32,
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
    pub fn expected(player: &Elo, opponent: &Elo) -> f32 {
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

        if opponent.rating == player.rating {
            0.5
        } else {
            let rating: f32 = (opponent.rating - player.rating) as f32 / ALGORITHM_OF;
            1f32 / (1f32 + 10f32.powf(rating))
        }
    }

    /// Update the rankings for the winner and loser.
    ///
    /// # Example
    ///
    /// ```
    /// let mut white = Elo::with_rating(800);
    /// let white_orig = white.clone();
    /// let mut black = Elo::new();
    /// Elo::update_ratings(&mut black, &mut white);
    /// ```
    pub fn update_ratings(winner: &mut Elo, loser: &mut Elo) {
        // The expected outcome for the loser will be `1 - expected outcome for winner`. This
        // allows us to simplify some math.
        let expected = Self::expected(&winner, &loser);

        // We're able to factor `1 - expected outcome` out from the following because
        // the equation for the loser ends up looking something like
        //
        // ```
        // Rn = Ra + K(actual outcome - expected outcome for loser)
        //    = Ra + K(0              - expected outcome for loser)
        //    = Ra + K(0              - (1 - expected outcome for winner))
        //    = Ra + K(0              - expected outcome for winner)
        // ```
        let winner_outcome = 1f32 - expected;
        winner.rating = (winner.rating as f32 + K_FACTOR * winner_outcome).floor() as i32;
        loser.rating = (loser.rating as f32 + K_FACTOR * (0f32 - winner_outcome)).floor() as i32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elo_expect_draw() {
        let white = Elo::new();
        let black = Elo::new();
        assert_eq!(Elo::expected(&white, &black), 0.5f32);
    }

    #[test]
    fn test_elo_expect_white_should_lose() {
        let white = Elo::with_rating(800);
        let black = Elo::new();
        assert!(Elo::expected(&white, &black) < 0.5f32);
    }

    #[test]
    fn test_elo_update_white_lose() {
        let mut white = Elo::with_rating(800);
        let white_orig = white.clone();
        let mut black = Elo::new();
        Elo::update_ratings(&mut black, &mut white);
        assert!(white.rating < white_orig.rating);
    }
}
