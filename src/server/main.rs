use tokio;
use axum::{
    routing::get,
    Router,
    Json,
};
use itertools::Itertools;
use std::collections::HashMap;
use tower_http::cors::CorsLayer;
use sqlx::SqlitePool;
use chrono::DateTime;
use magic_games_tracker::messages::*;
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
            start_datetime TEXT NOT NULL,
            end_datetime TEXT NOT NULL
            )").execute(&pool).await?;

    sqlx::query("CREATE TABLE IF NOT EXISTS games_players (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            game_id INTEGER NOT NULL,
            player_id INTEGER NOT NULL,
            commander TEXT NOT NULL,
            rank INTEGER NOT NULL,
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
        .route("/players", get({
                    let shared_state = Arc::clone(&shared_state);
                    || get_players(shared_state)
                })
               .post({
                        let shared_state = Arc::clone(&shared_state);
                        move |body| post_player(body, shared_state)
                    }))
        .layer(CorsLayer::permissive())
        .with_state(shared_state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn post_games(Json(payload): Json<CreateGamePayload>, state: Arc<AppState>) -> Json<Value> {
    if payload.players.len() < 2 {
        return Json(json!(
                    PostResponse {
                        success: false,
                        error: Some(String::from("A game must have at least two players"))
                    }
                ));
    }

    let num_draws = payload.players.iter().filter(|player| player.rank == 0).count();
    // If any player is marked as draw
    if num_draws != 0 && num_draws != payload.players.len() {
        return Json(json!(
            PostResponse {
                success: false,
                error: Some(String::from("If one player has drawn all players must have drawn"))
            }
        ));
    }

    let mut player_counts = HashMap::<String, i32>::new();

    for player in payload.players.iter() {
        player_counts.entry(player.name.clone()).and_modify(|counter| *counter += 1).or_insert(1);
    }

    for player_count in player_counts.into_values() {
        if player_count > 1 {
            return Json(json!(
                PostResponse {
                    success: false,
                    error: Some(String::from("Cannot have the same player multiple times"))
                }));
        }
    }

    let start_datetime = DateTime::parse_from_rfc3339(&payload.start_datetime).unwrap();
    let end_datetime = DateTime::parse_from_rfc3339(&payload.end_datetime).unwrap();


    if end_datetime <= start_datetime {
        return Json(json!(
            PostResponse {
                success: false,
                error: Some(String::from("End datetime cannot be earlier than or equal to start datetime"))
            }));
    }

    let sorted_ranks = payload.players
        .iter()
        .map(|player| player.rank)
        .sorted();

    // If all ranks are 1 that should be a marked a draw
    if sorted_ranks.clone().filter(|rank| (*rank).clone() == 1).count() == payload.players.len() {
        return Json(json!(
            PostResponse {
                success: false,
                error: Some(String::from("Cannot have all players ranks as 1. To mark a draw set all players rank to Draw"))
            }));

    }

    if num_draws == 0 {
        // The way this works is by taking the sorted ranks then ensuring that the first
        // rank is 1. After that we ensure the second rank is either equal to the
        // previous or equal to 2, then we check that third rank in order is equal to
        // 3 or the previous rank and so on.

        let enumerated_pairs = sorted_ranks
            .tuple_windows()
            .enumerate()
            .map(|(index, (prev, rank))| (index + 2, (prev, rank)))
            .collect::<Vec<(usize, (usize, usize))>>();

        // If prev in the first tuple is not 1 that's a problem
        if enumerated_pairs[0].1.0 != 1 {
            return Json(json!(
                PostResponse {
                    success: false,
                    error: Some(String::from("At least one player must come in first when there are no draws"))
                }));
        }

        // After the first prev is validated as being 1
        // we can compare cur to prev and the index of this pair
        // The only gotcha is that we have to start index at 2
        // because rankings start at 1 and due to the way tuple_windows
        // makes pairs the first cur is actually the second element in the list
        for (index, (prev, cur))in enumerated_pairs {
            if index != cur && prev != cur {
                return Json(json!(
                    PostResponse {
                        success: false,
                        error: Some(String::from(format!("Ranking is invalid player with a rank {} should have rank {} or {}", cur, index, prev)))
                    }));
            }
        }
    }

    for player in payload.players.iter() {
        if player.commander == "" {
            return Json(json!(
                PostResponse {
                    success: false,
                    error: Some(String::from("Player is missing a commander"))
                }));
        }
    }

    // END VALIDATION

    let row: (i64, ) = sqlx::query_as("INSERT INTO games (start_datetime, end_datetime) VALUES($1, $2) RETURNING id").bind(payload.start_datetime).bind(payload.end_datetime).fetch_one(&state.pool).await.unwrap();
    let game_id = row.0;

    for player in payload.players {
        let player_row_result: Result<(i64, ), sqlx::Error> = sqlx::query_as("SELECT id FROM players WHERE name = $1").bind(&player.name).fetch_one(&state.pool).await;

        match player_row_result {
            Ok(player_row) => {
                let player_id = player_row.0;
                sqlx::query("INSERT INTO games_players (game_id, player_id, commander, rank) VALUES($1, $2, $3, $4)").bind(game_id).bind(player_id).bind(player.commander).bind(player.rank as i32).execute(&state.pool).await.unwrap();
            },
            Err(error) => {
                return Json(json!({"error": error.to_string(), "success": false}));
            }
        }
    }

    Json(json!({ "success": true }))
}

async fn post_player(Json(payload): Json<PlayerPayload>, state: Arc<AppState>) -> Json<Value> {

    match sqlx::query("INSERT INTO players (name) VALUES($1)").bind(payload.name).execute(&state.pool).await {
        Ok(_) => Json(json!({ "success": true })),
        Err(error) if error.as_database_error().unwrap().code().unwrap() == "2067" => {
            Json(json!(
                    PostResponse{
                        success: false,
                        error: Some(String::from("Player already exists"))
                    }
                    ))
        }
        Err(error) => Json(json!(
                    PostResponse {
                        success: false,
                        error:  Some(error.to_string())
                    }
                ))
    }
}

async fn get_games(state: Arc<AppState>) -> Json<Value> {
    let mut games_response = GamesResponse{
        games: vec![]
    };

    let rows: Vec<(i64, String, String, String, String, i32)> = sqlx::query_as("SELECT games.id, start_datetime, end_datetime, players.name, commander, rank FROM games_players INNER JOIN games ON game_id = games.id INNER JOIN players ON player_id = players.id").fetch_all(&state.pool).await.unwrap();


    let ids = rows.iter().fold(Vec::new(), |mut acc, row| {
        if !acc.contains(&row.0) {
            acc.push(row.0);
        }

        acc
    });

    for id in ids {
        let game_rows: Vec<&(i64, String, String, String, String, i32)> = rows.iter().filter(|row| {
            row.0 == id
        }).collect();

        let mut players: Vec<Player> = Vec::new();
        let start_datetime: String = game_rows[0].1.clone();
        let end_datetime: String = game_rows[0].2.clone();
        for game_row in game_rows {
            players.push(Player{
                name: game_row.3.clone(),
                commander: game_row.4.clone(),
                rank: game_row.5 as usize
            })
        }

        let game = Game{
            start_datetime,
            end_datetime,
            players
        };

        games_response.games.push(game); 
    }


    Json(json!(games_response))
}

async fn get_players(state: Arc<AppState>) -> Json<Value> {
    let rows: Vec<(String,)> = sqlx::query_as("SELECT name FROM players").fetch_all(&state.pool).await.unwrap();

    // Flatten rows
    let names = rows.iter().fold(Vec::new(), |mut acc, row| {
        acc.push(row.0.clone());
        acc
    });

    let players_response = PlayersResponse{
        names
    };


    Json(json!(players_response))
}
