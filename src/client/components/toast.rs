use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub message: AttrValue,
}


#[function_component(Toast)]
pub fn toast(Props{ message }: &Props) -> Html {

    let hidden = use_state(|| false);

    let on_click = {
        let hidden = hidden.clone();
        Callback::from(move |_| {
            hidden.set(true);
        })
    };

    html! {
        <div onclick={on_click.clone()} hidden={*hidden} class="toast" >{message}</div>
    }
}
