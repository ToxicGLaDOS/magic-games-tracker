use gloo_net::http::Request;
use serde::{Serialize, Deserialize};
use web_sys::HtmlInputElement;
use wasm_bindgen::JsCast;
use ormos::messages::PostResponse;
use yew::prelude::*;

#[derive(Serialize, Deserialize)]
struct PlayerPayload {
    name: String
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub token: String,
    pub players_update_callback: Callback<String>,
    pub message_callback: Callback<String>
}

#[function_component(NewPlayerForm)]
pub fn new_player_form(Props{ token, players_update_callback, message_callback }: &Props) -> Html {
    let input_value = use_state(|| "".to_string());
    let message_callback = message_callback.clone();

    let onchange = {
        let input_value = input_value.clone();
        Callback::from(move |input_event: Event| {
            let input_event_target = input_event.target().unwrap();
            let current_input_text = input_event_target.unchecked_into::<HtmlInputElement>();

            input_value.set(current_input_text.value());
        })
    };

    let add_player = {
        let token = token.clone();
        let input_value = input_value.clone();
        let players_update_callback = players_update_callback.clone();
        Callback::from(move |_| {
            let token = token.clone();
            let input_value = input_value.clone();
            let message_callback = message_callback.clone();
            let players_update_callback = players_update_callback.clone();

            let payload = PlayerPayload{
                name: (*input_value).clone()
            };

            wasm_bindgen_futures::spawn_local(async move {
                let response: Result<PostResponse, gloo_net::Error> = Request::post("/api/players")
                    .header("Authorization", format!("Bearer {}", token.as_str()).as_str())
                    .json(&payload)
                    .unwrap()
                    .send()
                    .await
                    .unwrap()
                    .json()
                    .await;

                match response {
                    Ok(response) => {
                        if response.success {
                            players_update_callback.emit((*input_value).clone());
                            input_value.set("".to_string());
                        }
                        else {
                            message_callback.emit(response.error.unwrap());
                        }
                    },
                    Err(error) => {
                        message_callback.emit(format!("Server sent data we couldn't deserialze. Error was: {}", error.to_string()));
                    }
                }
            })

        })
    };


    html!{
        <>
        <label>{"Add new player"}</label>
        <input onchange={onchange.clone()} value={(*input_value).clone()}/>
        <button onclick={add_player.clone()}>{"Add player"}</button>
        </>
    }

}

