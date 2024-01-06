use gloo_console::log;
use web_sys::HtmlSelectElement;
use wasm_bindgen::JsCast;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub num_players: usize,
    pub select_callback: Callback<(usize, i32)>,
    pub index: usize
}

#[function_component(RankSelect)]
pub fn rank_select(Props{ num_players, select_callback, index }: &Props) -> Html {

    let on_change = {
        let index = index.clone();
        let select_callback = select_callback.clone();

        Callback::from(move |event: Event| {
            let mouse_event_target = event.target().unwrap();
            let select_element = mouse_event_target.unchecked_into::<HtmlSelectElement>();

            select_callback.emit((index, select_element.value().parse::<i32>().unwrap()));
        })
    };

    html!{
        <select onchange={on_change.clone()} class="rank-select">
            <option value={0} selected=true>{"Draw"}</option>
            {
                (0..4).map(|x| {
                    if x < *num_players {
                        html! {
                            <option value={(x+1).to_string()}>{(x+1).clone()}</option>
                        }
                    } else {
                        html! {}
                    }
                }).collect::<Html>()
            }
        </select>
    }
}

