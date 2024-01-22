use chrono::Duration;
use gloo_net::http::Request;
use wasm_bindgen_futures;
use gloo_console::log;
use web_sys::HtmlInputElement;
use wasm_bindgen::JsCast;
use gloo_timers::callback::Timeout;
use chrono::{Local, NaiveDateTime, DurationRound, TimeZone};
use ormos::messages::*;
use yew_hooks::prelude::*;
use crate::components::toast::*;
use crate::components::player_select::*;
use crate::components::rank_select::*;
use crate::components::commander_input::*;
use crate::components::player_data::*;
use yew::prelude::*;

fn create_message(messages: UseListHandle<String>, message: String) {
    messages.push(message);
    Timeout::new(5000, move || {
        messages.remove(0);
    }).forget();
}

#[derive(Clone)]
enum GameTime {
    Start,
    End
}

#[function_component(App)]
pub fn app() -> Html {

    let players = use_state(|| Vec::new());
    {
        let players = players.clone();
        use_effect_with((), move |_| {
            let players = players.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let fetched_players: PlayersResponse = Request::get("/api/players")
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

    let selected_players = use_state(|| ["".to_string(), "".to_string(), "".to_string(), "".to_string()]);

    let num_selected_players = selected_players.iter().filter(|player| *player != "").count();

    let on_player_select = |index: usize| {
        let selected_players = selected_players.clone();
        Callback::from(move |player: String| {
            let mut selected_players_copy = (*selected_players).clone();
            selected_players_copy[index] = player;
            selected_players.set(selected_players_copy);
        })
    };

    let selected_ranks = use_state(|| [0; 4]);

    let select_rank_callback = |index: usize| {
        let selected_ranks = selected_ranks.clone();
        Callback::from(move |rank: usize|{
            let mut copy = (*selected_ranks).clone();
            copy[index] = rank;
            selected_ranks.set(copy);
        })
    };

    let commander_inputs = use_state(|| ["".to_string(), "".to_string(), "".to_string(), "".to_string()]);

    let on_commander_input = |index: usize| {
        let commander_inputs = commander_inputs.clone();
        Callback::from(move |commander: String| {
            let mut commander_inputs_copy = (*commander_inputs).clone();
            commander_inputs_copy[index] = commander;
            commander_inputs.set(commander_inputs_copy);
        })
    };

    let partner_inputs = use_state(|| ["".to_string(), "".to_string(), "".to_string(), "".to_string()]);

    let on_partnet_input = |index: usize| {
        let partner_inputs = partner_inputs.clone();
        Callback::from(move |commander: String| {
            let mut partner_inputs_copy = (*partner_inputs).clone();
            partner_inputs_copy[index] = commander;
            partner_inputs.set(partner_inputs_copy);
        })
    };

    // "%Y-%m-%dT%H:%M"
    let start_datetime = use_state(|| Local::now().duration_round(Duration::minutes(1)).unwrap());
    let end_datetime = use_state(|| Local::now().duration_round(Duration::minutes(1)).unwrap());

    let datetime_oninput = { |game_time: GameTime| {
        let start_datetime = start_datetime.clone();
        let end_datetime = end_datetime.clone();
        Callback::from(move |event: InputEvent| {
            let game_time = game_time.clone();
            let input_event_target = event.target().unwrap();
            let mut current_input_text = input_event_target.unchecked_into::<HtmlInputElement>().value();

            // Set end to :00 so it can be converted to a DateTime properly
            current_input_text.push_str(":00");

            let from: NaiveDateTime = current_input_text.parse().unwrap();
            let date_time = Local.from_local_datetime(&from).unwrap();

            match game_time {
                GameTime::Start => start_datetime.set(date_time),
                GameTime::End => end_datetime.set(date_time)
            }
        })
    }};

    let messages = use_list(vec![]);

    let add_message = {
        let messages = messages.clone();
        Callback::from(move |m: String| {
            let messages = messages.clone();
            create_message(messages.clone(), m);
        })
    };

    let on_game_submit = {
        let messages = messages.clone();
        let commander_inputs = commander_inputs.clone();
        let selected_players = selected_players.clone();
        let selected_ranks = selected_ranks.clone();
        let start_datetime = start_datetime.clone();
        let end_datetime = end_datetime.clone();
        let mut players: Vec<Player> = Vec::new();
        for index in 0..4 {
            if selected_players[index] != "" {
                let mut commanders = Vec::new();
                if commander_inputs[index] != "" {
                    commanders.push(commander_inputs[index].clone());
                }

                if partner_inputs[index] != "" {
                    commanders.push(partner_inputs[index].clone());
                }

                players.push(
                    Player{
                        commanders,
                        name: selected_players[index].clone(),
                        rank: selected_ranks[index].clone()
                });
            }
        }

        let payload = CreateGamePayload{
            start_datetime: start_datetime.to_rfc3339(),
            end_datetime: end_datetime.to_rfc3339(),
            players
        };

        Callback::from(move |_| {
            let messages = messages.clone();
            let payload = payload.clone();
            log!(format!("{:?}", payload));

            wasm_bindgen_futures::spawn_local(async move {
                let response: PostResponse = Request::post("/api/games")
                    .json(&payload)
                    .unwrap()
                    .send()
                    .await
                    .unwrap()
                    .json()
                    .await
                    .unwrap();

                if !response.success {
                    create_message(messages.clone(), format!("Error: {}", response.error.unwrap()));
                } else {
                    create_message(messages.clone(), String::from("Game submitted successfully!"));
                }
            });
        })
    };

    let commanders = use_state(|| Vec::new());

    {
        let commanders = commanders.clone();
        use_effect_with((), move |_| {
            let commanders = commanders.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let fetched_commanders: CommandersResponse = Request::get("/api/commanders")
                    .send()
                    .await
                    .unwrap()
                    .json()
                    .await
                    .unwrap();
                commanders.set(fetched_commanders.commanders);
            });
        });
    }
    html! {
        <main>
            <table>
                <tr>
                    <td><label>{ "Start time" }</label></td>
                    <td><input type="datetime-local" oninput={datetime_oninput(GameTime::Start)} value={format!("{}", (*start_datetime).format("%Y-%m-%dT%H:%M"))} /></td>
                </tr>

                <tr>
                    <td><label>{ "End time" }</label></td>
                    <td><input type="datetime-local" oninput={datetime_oninput(GameTime::End)} value={format!("{}", (*end_datetime).format("%Y-%m-%dT%H:%M"))} /></td>
                </tr>
            </table>

			<datalist id="commanders">
				{
					commanders.iter().map(|commander: &String| { html! {
					<option value={(*commander).clone()}/>
                    }}).collect::<Html>()
				}
			</datalist>
            <table>
                <tr>
                    <td><label>{ "Players" }</label></td>
                    <td><label>{ "Commanders" }</label></td>
                    <td><label>{ "Partner" }</label></td>
                    <td><label>{ "Rank" }</label></td>
                </tr>
                <tr>
                    <td><PlayersSelect players={(*players).clone()} select_callback={on_player_select.clone()(0)}/></td>
                    <td><CommanderInput onchange={on_commander_input.clone()(0)}/></td>
                    <td><CommanderInput onchange={on_partnet_input.clone()(0)}/></td>
                    <td><RankSelect select_callback={select_rank_callback.clone()(0)} num_players={num_selected_players.clone()}/></td>
                </tr>
                <tr>
                    <td><PlayersSelect players={(*players).clone()} select_callback={on_player_select.clone()(1)}/></td>
                    <td><CommanderInput onchange={on_commander_input.clone()(1)}/></td>
                    <td><CommanderInput onchange={on_partnet_input.clone()(1)}/></td>
                    <td><RankSelect select_callback={select_rank_callback.clone()(1)} num_players={num_selected_players.clone()}/></td>
                </tr>
                <tr>
                    <td><PlayersSelect players={(*players).clone()} select_callback={on_player_select.clone()(2)}/></td>
                    <td><CommanderInput onchange={on_commander_input.clone()(2)}/></td>
                    <td><CommanderInput onchange={on_partnet_input.clone()(2)}/></td>
                    <td><RankSelect select_callback={select_rank_callback.clone()(2)} num_players={num_selected_players.clone()}/></td>
                </tr>
                <tr>
                    <td><PlayersSelect players={(*players).clone()} select_callback={on_player_select.clone()(3)}/></td>
                    <td><CommanderInput onchange={on_commander_input.clone()(3)}/></td>
                    <td><CommanderInput onchange={on_partnet_input.clone()(3)}/></td>
                    <td><RankSelect select_callback={select_rank_callback.clone()(3)} num_players={num_selected_players.clone()}/></td>
                </tr>
            </table>
            <button onclick={on_game_submit.clone()}>{"Submit"}</button>
            <br/>
            <NewPlayerForm players_update_callback={player_update_callback.clone()} message_callback={add_message}/>
            <div class="toast-container">
                {
                    messages.current().iter().map(|message| {
                        html!{
                            <Toast message={message.to_string()}/>
                        }
                    }).collect::<Html>()
                }
            </div>
        </main>
    }
}
