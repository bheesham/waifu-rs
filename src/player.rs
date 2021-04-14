use crate::elo::Elo;

#[derive(Debug, Clone)]
pub struct Player {
    pub name: String,
    pub elo: Elo,
}

impl Player {
    pub fn new(name: String, elo: Elo) -> Self {
        Self { name, elo }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_match() {
        let white = Player::new(String::from("white"), Elo::new());
        let black = Player::new(String::from("black"), Elo::new());
        assert_eq!(Elo::expected(&white.elo, &black.elo), 0.5f32);
    }
}
