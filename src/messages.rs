use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize)]
pub struct PlayersResponse{
    pub names: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct CommandersResponse{
    pub commanders: Vec<String>,
}

#[derive(Deserialize)]
pub struct BulkDataResponse {
    pub download_uri: String
}

#[derive(Serialize, Deserialize)]
pub struct PostResponse {
    pub success: bool,
    pub error: Option<String>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Player {
    pub name: String,
    pub commanders: Vec<String>,
    pub rank: usize
}

#[derive(Serialize, Deserialize)]
pub struct GamesResponse {
    pub games: Vec<Game>
}

#[derive(Serialize, Deserialize)]
pub struct Game {
    pub start_datetime: DateTime<Utc>,
    pub end_datetime: DateTime<Utc>,
    pub players: Vec<Player>,
}

#[derive(Serialize, Deserialize)]
pub struct PlayerPayload {
    pub name: String
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreateGamePayload {
    pub start_datetime: String,
    pub end_datetime: String,
    pub players: Vec<Player>,
}
