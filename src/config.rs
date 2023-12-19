use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub discord_token: String,
    pub database_path: String,
    pub accent_colour: u32,
}