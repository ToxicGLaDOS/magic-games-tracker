use tokio;
use axum::{
    routing::get,
    Router,
    Json,
};
use serde::{Serialize, Deserialize};
use sqlx::SqlitePool;
use sqlx::sqlite::SqliteConnectOptions;
use serde_json::{Value, json};
use std::sync::Arc;

struct AppState {
    pool: SqlitePool
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let options = SqliteConnectOptions::new()
        .create_if_missing(true)
        .filename("games.db");

    let pool = SqlitePool::connect_with(options)
        .await
        .expect("Could not connect to sqlite db");

    sqlx::query("CREATE TABLE IF NOT EXISTS players (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT UNIQUE NOT NULL
            )").execute(&pool).await?;

    sqlx::query("CREATE TABLE IF NOT EXISTS games (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            date TEXT NOT NULL
            )").execute(&pool).await?;

    sqlx::query("CREATE TABLE IF NOT EXISTS games_players (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            game_id INTEGER NOT NULL,
            player_id INTEGER NOT NULL,
            commander TEXT NOT NULL,
            winner BOOLEAN NOT NULL,
            FOREIGN KEY (game_id) REFERENCES games(id),
            FOREIGN KEY (player_id) REFERENCES players(id))").execute(&pool).await?;

    let shared_state = Arc::new(AppState { pool });

    // build our application with a single route
    let app = Router::new()
        .route("/games", get({
                    let shared_state = Arc::clone(&shared_state);
                    || get_games(shared_state)
                })
               .post({
                        let shared_state = Arc::clone(&shared_state);
                        move |body| post_games(body, shared_state)
                    }))
        .route("/players", get(get_players)
               .post({
                        let shared_state = Arc::clone(&shared_state);
                        move |body| post_player(body, shared_state)
                    }))
        .with_state(shared_state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

#[derive(Serialize, Deserialize)]
struct PlayerPayload {
    name: String
}

#[derive(Serialize, Deserialize)]
struct Player {
    name: String,
    commander: String,
    winner: bool
}

#[derive(Serialize, Deserialize)]
struct CreateGamePayload {
    date: String,
    players: Vec<Player>,
}

#[derive(Serialize, Deserialize)]
struct GamesResponse {
    games: Vec<Game>
}

#[derive(Serialize, Deserialize)]
struct Game {
    date: String,
    players: Vec<Player>,
}

async fn post_games(Json(payload): Json<CreateGamePayload>, state: Arc<AppState>) -> Json<Value> {
    let row: (i64, ) = sqlx::query_as("INSERT INTO games (date) VALUES($1) RETURNING id").bind(payload.date).fetch_one(&state.pool).await.unwrap();
    let game_id = row.0;

    for player in payload.players {
        let player_row_result: Result<(i64, ), sqlx::Error> = sqlx::query_as("SELECT id FROM players WHERE name = $1").bind(&player.name).fetch_one(&state.pool).await;

        match player_row_result {
            Ok(player_row) => {
                let player_id = player_row.0;
                sqlx::query("INSERT INTO games_players (game_id, player_id, commander, winner) VALUES($1, $2, $3, $4)").bind(game_id).bind(player_id).bind(player.commander).bind(player.winner).execute(&state.pool).await.unwrap();
            },
            Err(error) => {
                return Json(json!({"error": error.to_string(), "success": false}));
            }
        }
    }

    Json(json!({ "success": true }))
}

async fn post_player(Json(payload): Json<PlayerPayload>, state: Arc<AppState>) -> Json<Value> {
    sqlx::query("INSERT INTO players (name) VALUES($1)").bind(payload.name).execute(&state.pool).await.unwrap();

    Json(json!({ "success": true }))
}

async fn get_games(state: Arc<AppState>) -> Json<Value> {
    let mut games_response = GamesResponse{
        games: vec![]
    };

    let rows: Vec<(i64, String, String, String, i32)> = sqlx::query_as("SELECT games.id, date, players.name, commander, winner from games_players INNER JOIN games ON game_id = games.id INNER JOIN players ON player_id = players.id").fetch_all(&state.pool).await.unwrap();


    let ids = rows.iter().fold(Vec::new(), |mut acc, row| {
        if !acc.contains(&row.0) {
            acc.push(row.0);
        }

        acc
    });

    for id in ids {
        let game_rows: Vec<&(i64, String, String, String, i32)> = rows.iter().filter(|row| {
            row.0 == id
        }).collect();

        let mut players: Vec<Player> = Vec::new();
        let date: String = game_rows[0].1.clone();
        for game_row in game_rows {
            players.push(Player{
                name: game_row.2.clone(),
                commander: game_row.3.clone(),
                winner: game_row.4 == 1
            })
        }

        let game = Game{
            date,
            players
        };

        games_response.games.push(game); 
    }


    Json(json!(games_response))
}

async fn get_players() -> &'static str {
    "Get players"
}
