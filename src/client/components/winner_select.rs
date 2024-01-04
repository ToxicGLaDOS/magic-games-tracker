use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct WinnerSelectProps {
    pub chosen_players: Vec<String>,
    pub on_click: Callback<String>
}

#[function_component(WinnerSelect)]
pub fn winner_select(WinnerSelectProps { chosen_players, on_click }: &WinnerSelectProps) -> Html {
    let on_click = on_click.clone();
    let default_on_option_select = {
        let on_click = on_click.clone();
        Callback::from(move |_| {
            on_click.emit("Draw".to_string())
        })
    };


    html!{
        <select id="winner-select" class="winner-select">

        <option onclick={default_on_option_select} value="Draw" selected=true>{"Draw"}</option>
        {
            chosen_players.iter().map(|player| {
                let on_option_select = {
                    let on_click = on_click.clone();
                    let player = player.clone();
                    Callback::from(move |_| {
                        on_click.emit(player.clone())
                    })
                };

                if player != "" {
                    html!{
                    <option onclick={on_option_select} value={player.clone()}>{player.clone()}</option>
                    }
                } else {
                    html!{}
                }
            }).collect::<Html>()
        }

        </select>
    }
}

