use gloo_net::http::Request;
use serde::{Serialize, Deserialize};
use web_sys::HtmlInputElement;
use wasm_bindgen::JsCast;
use yew::prelude::*;

#[derive(Deserialize)]
struct PostResponse {
    success: bool,
    error: Option<bool>
}

#[derive(Serialize, Deserialize)]
struct PlayerPayload {
    name: String
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub players_update_callback: Callback<String>
}

#[function_component(NewPlayerForm)]
pub fn new_player_form(Props{ players_update_callback }: &Props) -> Html {
    let input_value = use_state(|| "".to_string());

    let onchange = {
        let input_value = input_value.clone();
        Callback::from(move |input_event: Event| {
            let input_event_target = input_event.target().unwrap();
            let current_input_text = input_event_target.unchecked_into::<HtmlInputElement>();

            input_value.set(current_input_text.value());
        })
    };

    let add_player = {
        let input_value = input_value.clone();
        let players_update_callback = players_update_callback.clone();
        Callback::from(move |_| {
            let input_value = input_value.clone();
            let players_update_callback = players_update_callback.clone();

            let payload = PlayerPayload{
                name: (*input_value).clone()
            };

            wasm_bindgen_futures::spawn_local(async move {
                let response: PostResponse = Request::post("http://localhost:3000/players")
                    .json(&payload)
                    .unwrap()
                    .send()
                    .await
                    .unwrap()
                    .json()
                    .await
                    .unwrap();

                if response.success {
                    players_update_callback.emit((*input_value).clone());
                    input_value.set("".to_string());
                }
                // TODO: Else show an error
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

