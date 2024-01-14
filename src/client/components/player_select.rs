use web_sys::HtmlSelectElement;
use wasm_bindgen::JsCast;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct PlayersSelectProps {
    pub players: Vec<String>,
    pub select_callback: Callback<String>
}

#[function_component(PlayersSelect)]
pub fn players_select(PlayersSelectProps { players, select_callback }: &PlayersSelectProps) -> Html {

    let on_change = {
        let select_callback = select_callback.clone();
        Callback::from(move |event: Event| {
            let event_target = event.target().unwrap();
            let select_element = event_target.unchecked_into::<HtmlSelectElement>();

            select_callback.emit(select_element.value());
        })
    };

    html!{
        <select onchange={on_change.clone()} class="player-select">

        <option value=""></option>
        {
            players.iter().map(|player| {

                html!{
                    <option key={player.clone()} value={player.clone()}>{player.clone()}</option>
                }
            }
            ).collect::<Html>()
        }

        </select>
    }
}
