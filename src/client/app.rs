use gloo_net::http::Request;
use wasm_bindgen_futures;
use serde::Deserialize;
use gloo_console::log;
use yew::prelude::*;

#[derive(Clone, PartialEq, Deserialize)]
struct Player {
    name: String
}

#[derive(Properties, PartialEq)]
struct PlayersSelectProps {
    players: Vec<String>,
    on_click: Callback<String>
}

#[derive(Properties, PartialEq)]
struct WinnerSelectProps {
    chosen_players: Vec<String>,
}

#[derive(Deserialize)]
struct PlayersResponse{
    names: Vec<String>,
}

#[function_component(PlayersSelect)]
fn players_select(PlayersSelectProps { players, on_click }: &PlayersSelectProps) -> Html {
    let on_click = on_click.clone();
    let default_on_player_select = {
        let on_click = on_click.clone();
        Callback::from(move |_| {
            on_click.emit("".to_string())
        })
    };

    html!{
        <select class="player-select">

        <option onclick={default_on_player_select} value=""></option>
        {
            players.iter().map(|player| {
                let on_player_select = {
                    let on_click = on_click.clone();
                    let player = player.clone();
                    Callback::from(move |_| {
                        on_click.emit(player.clone())
                    })
                };

                html!{
                    <option onclick={on_player_select} value={player.clone()}>{player.clone()}</option>
                }
            }
            ).collect::<Html>()
        }

        </select>
    }
}

#[function_component(WinnerSelect)]
fn winner_select(WinnerSelectProps { chosen_players }: &WinnerSelectProps) -> Html {
    html!{
        <select id="winner-select" class="winner-select">

        <option value="Draw" selected=true>{"Draw"}</option>
        {
            chosen_players.iter().map(|player|
                if player != "" {
                    html!{
                    <option value={player.clone()}>{player.clone()}</option>
                    }
                } else {
                    html!{}
                }
            ).collect::<Html>()
        }

        </select>
    }
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
            log!("Log");
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
                    <td><input id="commander1-input" class="commander-input"/></td>
                    <td><input id="commander2-input" class="commander-input"/></td>
                    <td><input id="commander3-input" class="commander-input"/></td>
                    <td><input id="commander4-input" class="commander-input"/></td>
                </tr>
                <tr>
                    <label>{ "Winner" }</label>
                    <td><WinnerSelect chosen_players={(*selected_players).clone()} /></td>
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
