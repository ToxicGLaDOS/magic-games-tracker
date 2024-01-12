use web_sys::HtmlInputElement;
use wasm_bindgen::JsCast;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct CommanderInputProps {
    pub onchange: Callback<String>
}

#[function_component(CommanderInput)]
pub fn commander_input(CommanderInputProps{ onchange }: &CommanderInputProps) -> Html {
    let onchange = onchange.clone();

    let handle_onchange = Callback::from(

        move | input_event: Event | {
            let input_event_target = input_event.target().unwrap();
            let current_input_text = input_event_target.unchecked_into::<HtmlInputElement>();

            onchange.emit(current_input_text.value());
        }

    );
    html!{
        <input list="commanders" class="commander-input" onchange={handle_onchange}/>
    }
}
