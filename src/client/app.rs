use gloo_net::http::Request;
use wasm_bindgen_futures;
use serde::{Serialize, Deserialize};
use gloo_console::log;
use crate::components::player_select::*;
use crate::components::winner_select::*;
use crate::components::commander_input::*;
use crate::components::player_data::*;
use yew::prelude::*;

#[derive(Deserialize)]
struct PlayersResponse{
    names: Vec<String>,
}

#[derive(Deserialize)]
struct PostResponse {
    success: bool,
    error: Option<bool>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Player {
    name: String,
    commander: String,
    winner: bool
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct CreateGamePayload {
    date: String,
    players: Vec<Player>,
}

#[function_component(App)]
pub fn app() -> Html {

    let player_add_counter = use_state(|| 0);


    let players = use_state(|| Vec::new());
    {
        let players = players.clone();
        use_effect_with((), move |_| {
            let players = players.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let fetched_players: PlayersResponse = Request::get("http://localhost:3000/players")
                    .send()
                    .await
                    .unwrap()
                    .json()
                    .await
                    .unwrap();
                players.set(fetched_players.names);
            });
        });
    }

    let player_update_callback = {
        let players = players.clone();

        Callback::from(move |player: String| {
            let players = players.clone();
            let mut new_players = (*players).clone();
            new_players.push(player);
            players.set(new_players);
        })
    };

    let selected_players = use_state(|| vec!["".to_string(), "".to_string(), "".to_string(), "".to_string()]);

    let on_player_select = |index: usize| {
        let selected_players = selected_players.clone();
        Callback::from(move |player: String| {
            let mut new_selected_players: Vec<String> = Vec::new();
            for (i, selected_player) in selected_players.iter().enumerate() {
                if i == index {
                    new_selected_players.push(player.clone());
                }
                else {
                    new_selected_players.push((*selected_player).clone());
                }
            }
            selected_players.set(new_selected_players);
        })
    };


    let commander_inputs = use_state(|| vec!["".to_string(), "".to_string(), "".to_string(), "".to_string()]);

    let on_commander_input = |index: usize| {
        let commander_inputs = commander_inputs.clone();
        Callback::from(move |commander: String| {
            let mut new_commander_inputs: Vec<String> = Vec::new();
            for (i, commander_input) in commander_inputs.iter().enumerate() {
                if i == index {
                    new_commander_inputs.push(commander.clone());
                }
                else {
                    new_commander_inputs.push((*commander_input).clone());
                }
            }
            commander_inputs.set(new_commander_inputs);
        })
    };

    let winner_selection = use_state(|| "Draw".to_string());

    let on_winner_select = {
        let winner_selection = winner_selection.clone();
        Callback::from(move |winner: String| {
            winner_selection.set(winner);
        })
    };


    let on_game_submit = {
        let commander_inputs = commander_inputs.clone();
        let selected_players = selected_players.clone();
        let winner = winner_selection.clone();
        let date = "1/3/2024".to_string();
        let mut players: Vec<Player> = Vec::new();
        for index in 0..4 {
            if selected_players[index] != "" {
                players.push(
                    Player{
                        commander: commander_inputs[index].clone(),
                        name: selected_players[index].clone(),
                        winner: *winner == selected_players[index].clone()
                });
            }
        }

        let payload = CreateGamePayload{
            date,
            players
        };

        Callback::from(move |_| {
            let payload = payload.clone();
            log!(format!("{:?}", payload));
            wasm_bindgen_futures::spawn_local(async move {
                let response: PostResponse = Request::post("http://localhost:3000/games")
                    .json(&payload)
                    .unwrap()
                    .send()
                    .await
                    .unwrap()
                    .json()
                    .await
                    .unwrap();

                if !response.success {
                    panic!("Failed to POST to /games");
                }
            });
        })
    };


    html! {
        <main>
            <table>
                <tr>
                    <td><label>{ "Players" }</label></td>
                    <td><label>{ "Commanders" }</label></td>
                </tr>
                <tr>
                    <td><PlayersSelect players={(*players).clone()} on_click={on_player_select.clone()(0)}/></td>
                    <td><CommanderInput onchange={on_commander_input.clone()(0)}/></td>
                </tr>
                <tr>
                    <td><PlayersSelect players={(*players).clone()} on_click={on_player_select.clone()(1)}/></td>
                    <td><CommanderInput onchange={on_commander_input.clone()(1)}/></td>
                </tr>
                <tr>
                    <td><PlayersSelect players={(*players).clone()} on_click={on_player_select.clone()(2)}/></td>
                    <td><CommanderInput onchange={on_commander_input.clone()(2)}/></td>
                </tr>
                <tr>
                    <td><PlayersSelect players={(*players).clone()} on_click={on_player_select.clone()(3)}/></td>
                    <td><CommanderInput onchange={on_commander_input.clone()(3)}/></td>
                </tr>
                <tr>
                    <td><label>{ "Winner" }</label></td>
                    <td><WinnerSelect chosen_players={(*selected_players).clone()} on_click={on_winner_select} /></td>
                </tr>
            </table>
            <button onclick={on_game_submit.clone()}>{"Submit"}</button>
            <br/>
            <PlayerData players_update_callback={player_update_callback.clone()}/>
        </main>
    }
}
