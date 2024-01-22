use tokio;
use axum::{
    routing::get,
    Router,
    Json,
    http::StatusCode, response::IntoResponse
};
use reqwest;
use reqwest::Method;
use std::{env, thread, fs, fs::File, path::Path, io::Write, time::Duration, sync::Arc, collections::HashMap};
use itertools::Itertools;
use tower_http::{cors::CorsLayer, services::ServeDir};
use chrono::{DateTime, Utc};
use clap::Parser;
use ormos::messages::*;
use sqlx::postgres::{PgPoolOptions, PgPool};
use serde::Deserialize;

struct AppState {
    pool: PgPool
}

#[derive(Parser, Debug)]
struct CliOptions {
    /// set the listen addr
    #[clap(short = 'a', long = "addr", default_value = "127.0.0.1")]
    addr: String,

    /// set the listen port
    #[clap(short = 'p', long = "port", default_value = "8081")]
    port: u16,

    /// set the directory where static files are to be found
    #[clap(long = "static-dir", default_value = "./dist")]
    static_dir: String,
}

#[derive(Deserialize)]
struct ScryfallLegalities {
    commander: String
}

#[derive(Deserialize)]
struct ScryfallPart {
    component: String,
    name: String
}

#[derive(Deserialize)]
struct ScryfallCardFace {
    type_line: Option<String>,
    name: String
}

#[derive(Deserialize)]
struct ScryfallCard {
    type_line: Option<String>,
    name: String,
    legalities: ScryfallLegalities,
    games: Vec<String>,
    oracle_text: Option<String>,
    all_parts: Option<Vec<ScryfallPart>>,
    card_faces: Option<Vec<ScryfallCardFace>>
}

fn generate_commanders() {
    println!("Getting bulk data URI");
    let bulk_data_response: BulkDataResponse = reqwest::blocking::get("https://api.scryfall.com/bulk-data/default-cards")
        .unwrap()
        .json()
        .unwrap();

    println!("Getting bulk data from {}", bulk_data_response.download_uri);
    let cards: Vec<ScryfallCard> = reqwest::blocking::Client::new()
        .request(Method::GET, bulk_data_response.download_uri)
        .timeout(Duration::from_secs(60 * 20))
        .send()
        .unwrap()
        .json()
        .unwrap();

    let mut commanders: Vec<String> = Vec::new();

    for card in cards {
        let mut type_line = card.type_line;
        let mut name = card.name;

        if card.legalities.commander != "legal" {
            continue
        }

        if !card.games.contains(&String::from("paper")) {
            continue
        }

        // Skip stuff like Brisela
        if let Some(parts) = card.all_parts {
            let meld_result_parts: Vec<&ScryfallPart> = parts.iter().filter(|part| part.component == "meld_result").collect();
            if !meld_result_parts.is_empty() {
                if meld_result_parts[0].name == *name {
                    continue
                }
            }
        }

        if let Some(faces) = card.card_faces {
            if let Some(inner_type_line) = &faces[0].type_line {
                type_line = Some(inner_type_line.clone());
            }
            name = faces[0].name.clone();
        }


        if let Some(type_line) = type_line {
            if commanders.contains(&name) {
                continue
            }
            if type_line.contains("Creature") && type_line.contains("Legendary") {
                commanders.push(name);
            }
            else if type_line.contains("Background") {
                commanders.push(name);
            }
            else if name == "Grist, the Hunger Tide" {
                commanders.push(name);
            }
            else if let Some(oracle_text) = card.oracle_text {
                if oracle_text.contains("can be your commander") {
                    commanders.push(name);
                }
            }
        }
    }

    commanders.sort();

    let mut file = File::create("commanders.json").unwrap();
    let s = serde_json::to_string(&commanders).unwrap();
    write!(file, "{}", s).unwrap();

    println!("Loaded {} commanders", commanders.len());
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let _commander_thread = thread::spawn(|| {
        // This is just to prevent downloading every time
        // the program runs during development, but still
        // allows a fresh server to download immediately
        if !Path::new("commanders.json").exists() {
            generate_commanders();
        }
        loop {
            thread::sleep(Duration::from_secs(60 * 60 * 24));
            generate_commanders();
        }
    });

    let opts = CliOptions::parse();

    // Defaults values correspond to development postgres, not production
    let pg_user = env::var("POSTGRES_USER").unwrap_or(String::from("postgres"));
    let pg_password = env::var("POSTGRES_PASSWORD").unwrap_or(String::from("password"));
    let pg_host = env::var("POSTGRES_HOST").unwrap_or(String::from("localhost"));
    let pg_port = env::var("POSTGRES_PORT").unwrap_or(String::from("55432"));
    let pg_database = env::var("POSTGRES_DB").unwrap_or(String::from("ormos"));

    let connection_string = format!("postgres://{}:{}@{}:{}/{}", pg_user, pg_password, pg_host, pg_port, pg_database);

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(connection_string.as_str()).await?;

    sqlx::query("CREATE TABLE IF NOT EXISTS players (
            id SERIAL PRIMARY KEY,
            name TEXT UNIQUE NOT NULL
            )").execute(&pool).await?;

    sqlx::query("CREATE TABLE IF NOT EXISTS games (
            id SERIAL PRIMARY KEY,
            start_datetime TIMESTAMP WITH TIME ZONE NOT NULL,
            end_datetime TIMESTAMP WITH TIME ZONE NOT NULL
            )").execute(&pool).await?;

    sqlx::query("CREATE TABLE IF NOT EXISTS games_players (
            id SERIAL PRIMARY KEY,
            game_id INTEGER NOT NULL,
            player_id INTEGER NOT NULL,
            rank INTEGER NOT NULL,
            FOREIGN KEY (game_id) REFERENCES games(id),
            FOREIGN KEY (player_id) REFERENCES players(id))").execute(&pool).await?;

    sqlx::query("CREATE TABLE IF NOT EXISTS commanders (
            id SERIAL PRIMARY KEY,
            games_players_id INTEGER NOT NULL,
            commander TEXT NOT NULL,
            FOREIGN KEY (games_players_id) REFERENCES games_players(id)
            )").execute(&pool).await?;

    let shared_state = Arc::new(AppState { pool });

    // build our application with a single route
    let app = Router::new()
        .route("/api/games", get({
                    let shared_state = Arc::clone(&shared_state);
                    || get_games(shared_state)
                })
               .post({
                        let shared_state = Arc::clone(&shared_state);
                        move |body| post_games(body, shared_state)
                    }))
        .route("/api/players", get({
                    let shared_state = Arc::clone(&shared_state);
                    || get_players(shared_state)
                })
               .post({
                        let shared_state = Arc::clone(&shared_state);
                        move |body| post_player(body, shared_state)
                    }))
        .route("/api/commanders", get(get_commanders))
        .layer(CorsLayer::permissive())
        .fallback_service(
            ServeDir::new(opts.static_dir)
            )
        .with_state(shared_state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", opts.addr, opts.port)).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn post_games(Json(payload): Json<CreateGamePayload>, state: Arc<AppState>) -> impl IntoResponse {
    if payload.players.len() < 2 {
        return (StatusCode::BAD_REQUEST, Json(
                    PostResponse {
                        success: false,
                        error: Some(String::from("A game must have at least two players"))
                    }
                ));
    }

    let mut player_counts = HashMap::<String, i32>::new();

    for player in payload.players.iter() {
        player_counts.entry(player.name.clone()).and_modify(|counter| *counter += 1).or_insert(1);
    }

    for player_count in player_counts.into_values() {
        if player_count > 1 {
            return (StatusCode::BAD_REQUEST, Json(
                PostResponse {
                    success: false,
                    error: Some(String::from("Cannot have the same player multiple times"))
                }));
        }
    }

    let start_datetime = DateTime::parse_from_rfc3339(&payload.start_datetime).unwrap();
    let end_datetime = DateTime::parse_from_rfc3339(&payload.end_datetime).unwrap();


    if end_datetime <= start_datetime {
        return (StatusCode::BAD_REQUEST, Json(
            PostResponse {
                success: false,
                error: Some(String::from("End datetime cannot be earlier than or equal to start datetime"))
            }));
    }

    let sorted_ranks = payload.players
        .iter()
        .map(|player| player.rank)
        .sorted();

    // If any ranks are outside the bounds of 1-<number of players>
    // then the ranking is invalid
    for player in payload.players.iter() {
        if player.rank < 1 || player.rank > payload.players.len() {
            return (StatusCode::BAD_REQUEST, Json(
                PostResponse {
                    success: false,
                    error: Some(format!("Player {} has invalid value for rank", player.name))
                }));
        }
    }

    // The way we validate ranks is by ensuring that the first in sorted_ranks
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
        return (StatusCode::BAD_REQUEST, Json(
            PostResponse {
                success: false,
                error: Some(String::from("At least one player must come in first"))
            }));
    }

    // After the first prev is validated as being 1
    // we can compare cur to prev and the index of this pair
    // The only gotcha is that we have to start index at 2
    // because rankings start at 1 and due to the way tuple_windows
    // makes pairs the first cur is actually the second element in the list
    for (index, (prev, cur)) in enumerated_pairs {
        if index != cur && prev != cur {
            return (StatusCode::BAD_REQUEST, Json(
                PostResponse {
                    success: false,
                    error: Some(String::from(format!("Ranking is invalid player with a rank {} should have rank {} or {}", cur, index, prev)))
                }));
        }
    }

    for player in payload.players.iter() {
        if player.commanders.is_empty() {
            return (StatusCode::BAD_REQUEST, Json(
                    PostResponse {
                        success: false,
                        error: Some(format!("Player \"{}\" has no commanders", player.name))
                    }));
        }
        for commander in player.commanders.clone() {
            if commander == "" {
                return (StatusCode::BAD_REQUEST, Json(
                    PostResponse {
                        success: false,
                        error: Some(format!("Player \"{}\" has an empty string as a commander", player.name))
                    }));
            }
        }
    }

    // END VALIDATION

    let mut tx = state.pool.begin().await.unwrap();

    let row: (i32, ) = sqlx::query_as("INSERT INTO games (start_datetime, end_datetime) VALUES($1, $2) RETURNING id").bind(start_datetime).bind(end_datetime).fetch_one(&mut *tx).await.unwrap();
    let game_id = row.0;

    for player in payload.players {
        let player_row_result: Result<(i32, ), sqlx::Error> = sqlx::query_as("SELECT id FROM players WHERE name = $1").bind(&player.name).fetch_one(&mut *tx).await;

        match player_row_result {
            Ok(player_row) => {
                let player_id = player_row.0;
                // TODO: Handle potential error instead of unwrapping
                let row: (i32,) = sqlx::query_as("INSERT INTO games_players (game_id, player_id, rank) VALUES($1, $2, $3) RETURNING id").bind(game_id).bind(player_id).bind(player.rank as i32).fetch_one(&mut *tx).await.unwrap();
                let games_players_id = row.0;
                for commander in player.commanders {
                    sqlx::query("INSERT INTO commanders (games_players_id, commander) VALUES($1, $2)").bind(games_players_id).bind(commander).execute(&mut *tx).await.unwrap();
                }
            },
            Err(error) => {
                return (StatusCode::BAD_REQUEST, Json(PostResponse{ 
                    success: false,
                    error: Some(error.to_string())
                }));
            }
        }
    }
    tx.commit().await.unwrap();

    (StatusCode::OK, Json(PostResponse { success:true, error: None }))
}

async fn post_player(Json(payload): Json<PlayerPayload>, state: Arc<AppState>) -> impl IntoResponse {

    match sqlx::query("INSERT INTO players (name) VALUES($1)").bind(payload.name).execute(&state.pool).await {
        Ok(_) => (StatusCode::OK, Json(PostResponse { success: true, error: None})),
        Err(error) if error.as_database_error().unwrap().code().unwrap() == "23505" => {
            (StatusCode::BAD_REQUEST, Json(
                    PostResponse{
                        success: false,
                        error: Some(String::from("Player already exists"))
                    }
                    ))
        }
        Err(error) => (StatusCode::BAD_REQUEST, Json(
                    PostResponse {
                        success: false,
                        error:  Some(error.to_string())
                    }
                ))
    }
}

async fn get_games(state: Arc<AppState>) -> Json<GamesResponse> {
    let mut games_response = GamesResponse{
        games: vec![]
    };

    let rows: Vec<(i32, i32, DateTime<Utc>, DateTime<Utc>, String, i32)> = sqlx::query_as("SELECT games.id, games_players.id, start_datetime, end_datetime, players.name, rank FROM games_players INNER JOIN games ON game_id = games.id INNER JOIN players ON player_id = players.id").fetch_all(&state.pool).await.unwrap();

    let commander_rows: Vec<(i32, String)> = sqlx::query_as("SELECT games_players.id, commander FROM commanders INNER JOIN games_players ON games_players_id = games_players.id").fetch_all(&state.pool).await.unwrap();

    let unique_game_ids = rows.iter().fold(Vec::new(), |mut acc, row| {
        if !acc.contains(&row.0) {
            acc.push(row.0);
        }

        acc
    });

    let games_players_id_to_commanders = commander_rows.iter().fold(HashMap::new(), |mut acc: HashMap<i32, Vec<String>>, row| {
        match acc.get_mut(&row.0) {
            Some(commanders) => {
                commanders.push(row.1.clone());
            },
            None => {
                acc.insert(row.0, vec![row.1.clone()]);
            }
        }

        acc
    });

    for id in unique_game_ids {
        let game_rows: Vec<&(i32, i32, DateTime<Utc>, DateTime<Utc>, String, i32)> = rows.iter().filter(|row| {
            row.0 == id
        }).collect();

        let mut players: Vec<Player> = Vec::new();
        let start_datetime = game_rows[0].2.clone();
        let end_datetime = game_rows[0].3.clone();

        for game_row in game_rows {
            players.push(Player{
                name: game_row.4.clone(),
                commanders: games_players_id_to_commanders.get(&game_row.1).unwrap().clone(),
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


    Json(games_response)
}

async fn get_players(state: Arc<AppState>) -> Json<PlayersResponse> {
    let rows: Vec<(String,)> = sqlx::query_as("SELECT name FROM players").fetch_all(&state.pool).await.unwrap();

    // Flatten rows
    let names = rows.iter().fold(Vec::new(), |mut acc, row| {
        acc.push(row.0.clone());
        acc
    });

    let players_response = PlayersResponse{
        names
    };


    Json(players_response)
}

async fn get_commanders() -> Json<CommandersResponse> {
    let json_text = fs::read_to_string("commanders.json").unwrap();
    let cards: Vec<String> = serde_json::from_str(&json_text).expect("JSON was not well-formatted");

    Json(CommandersResponse {
        commanders: cards
    })
}
