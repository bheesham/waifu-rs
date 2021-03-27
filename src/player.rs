use crate::elo::Elo;

#[derive(Debug, Clone)]
pub struct Player {
    pub name: String,
    pub rating: Elo,
}

impl Player {
    pub fn new(name: String, rating: Elo) -> Self {
        Self { name, rating }
    }
}

#[test]
fn test_player_match() {
    let white = Player::new(String::from("white"), Elo::new());
    let black = Player::new(String::from("black"), Elo::new());
    assert_eq!(Elo::expected(&white.rating, &black.rating), 0.5f32);
}
