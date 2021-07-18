use std::env;

#[derive(Default)]
pub struct Config {
    pub username: String,
    pub password: String,
    pub file_db: String,
    pub url_index: String,
    pub url_login: String,
    pub url_state: String,
    pub url_bet: String,
    pub url_referer: String,
}

/// Reads variables from the environment and populates the `Config` struct.
pub fn configure() -> Config {
    let file_db = env::var("W_FILE_PATH").unwrap_or(String::from("memory"));
    let url_index = env::var("SB_INDEX_URL").unwrap_or(String::from("https://www.saltybet.com/"));
    let url_state =
        env::var("SB_STATE_URL").unwrap_or(String::from("https://www.saltybet.com/state.json"));
    let url_login = env::var("SB_LOGIN_URL").unwrap_or(String::from(
        "https://www.saltybet.com/authenticate?signin=1",
    ));
    let url_bet = env::var("SB_BET_URL")
        .unwrap_or(String::from("http://www.saltybet.com/ajax_place_bet.php"));
    let url_referer =
        env::var("SB_REFERER_URL").unwrap_or(String::from("http://www.saltybet.com/"));

    let username = env::var("SB_USERNAME").expect("No SB_USERNAME environment variable supplied.");
    let password = env::var("SB_PASSWORD").expect("No SB_PASSWORD environment variable supplied.");

    Config {
        username,
        password,
        file_db,
        url_index,
        url_login,
        url_state,
        url_bet,
        url_referer,
    }
}
