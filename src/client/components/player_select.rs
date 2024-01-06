use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct PlayersSelectProps {
    pub players: Vec<String>,
    pub on_click: Callback<String>
}

#[function_component(PlayersSelect)]
pub fn players_select(PlayersSelectProps { players, on_click }: &PlayersSelectProps) -> Html {

    let on_click = move |player: String| {
        let on_click = on_click.clone();
        Callback::from(move |_| {
            let player = player.clone();
            on_click.emit(player);
        })
    };

    html!{
        <select class="player-select">

        <option onclick={on_click("".to_string())} value=""></option>
        {
            players.iter().map(|player| {

                html!{
                    <option onclick={on_click(player.clone())} value={player.clone()}>{player.clone()}</option>
                }
            }
            ).collect::<Html>()
        }

        </select>
    }
}
