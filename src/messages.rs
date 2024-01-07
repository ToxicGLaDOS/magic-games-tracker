use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct PlayersResponse{
    pub names: Vec<String>,
}

#[derive(Deserialize)]
pub struct PostResponse {
    pub success: bool,
    pub error: Option<bool>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Player {
    pub name: String,
    pub commander: String,
    pub rank: usize
}

#[derive(Serialize, Deserialize)]
pub struct GamesResponse {
    pub games: Vec<Game>
}

#[derive(Serialize, Deserialize)]
pub struct Game {
    pub start_datetime: String,
    pub end_datetime: String,
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
