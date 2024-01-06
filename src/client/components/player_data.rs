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

}

#[function_component(PlayerData)]
pub fn player_data(Props{ }: &Props) -> Html { 
    let player_name_input_value = use_state(|| "".to_string());
    let input_value = player_name_input_value.clone();

    let onchange = Callback::from(move |input_event: Event| {
        let input_event_target = input_event.target().unwrap();
        let current_input_text = input_event_target.unchecked_into::<HtmlInputElement>();

        player_name_input_value.set(current_input_text.value());
    });

    let add_player = Callback::from(move |_| {
        let player_name_input_value = input_value.clone();

        let payload = PlayerPayload{
            name: (*player_name_input_value).clone()
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
        })

    });


    html!{
        <>
        <label>{"Add new player"}</label>
        <input onchange={onchange.clone()}/>
        <button onclick={add_player.clone()}>{"Add player"}</button>
        </>
    }

}

