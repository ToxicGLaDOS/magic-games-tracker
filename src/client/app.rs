use gloo_net::http::Request;
use wasm_bindgen_futures;
use serde::Deserialize;
use gloo_console::log;
use crate::components::player_select::*;
use crate::components::winner_select::*;
use crate::components::commander_input::*;
use yew::prelude::*;

#[derive(Deserialize)]
struct PlayersResponse{
    names: Vec<String>,
}

#[function_component(App)]
pub fn app() -> Html {

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
            || ()
        });
    }

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

    let winner_selection = use_state(|| None);

    let on_winner_select = {
        let winner_selection = winner_selection.clone();
        Callback::from(move |winner: String| {
            winner_selection.set(Some(winner));
        })
    };


    //let game_submit = {
    //    let commander_inputs = commander_inputs.clone();
    //    let selected_players = selected_players.clone();
    //    let winner
    //}


    html! {
        <main>
            <table>
                <tr>
                    <td><label>{ "Players" }</label></td>
                    <td><PlayersSelect players={(*players).clone()} on_click={on_player_select.clone()(0)}/></td>
                    <td><PlayersSelect players={(*players).clone()} on_click={on_player_select.clone()(1)}/></td>
                    <td><PlayersSelect players={(*players).clone()} on_click={on_player_select.clone()(2)}/></td>
                    <td><PlayersSelect players={(*players).clone()} on_click={on_player_select.clone()(3)}/></td>
                </tr>
                <tr>
                    <label>{ "Commanders" }</label>
                    <td><CommanderInput onchange={on_commander_input.clone()(0)}/></td>
                    <td><CommanderInput onchange={on_commander_input.clone()(1)}/></td>
                    <td><CommanderInput onchange={on_commander_input.clone()(2)}/></td>
                    <td><CommanderInput onchange={on_commander_input.clone()(3)}/></td>
                </tr>
                <tr>
                    <label>{ "Winner" }</label>
                    <td><WinnerSelect chosen_players={(*selected_players).clone()} on_click={on_winner_select} /></td>
                </tr>
            </table>
            <input type="submit" value="Submit"/>
            <br/>
            <label>{"Add new player"}</label>
            <input id="new-player-input" class="new-player-input"/>
            <input type="submit" value="Submit"/>
        </main>
    }
}
