use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct PlayersSelectProps {
    pub players: Vec<String>,
    pub on_click: Callback<String>
}

#[function_component(PlayersSelect)]
pub fn players_select(PlayersSelectProps { players, on_click }: &PlayersSelectProps) -> Html {
    let on_click = on_click.clone();
    let default_on_player_select = {
        let on_click = on_click.clone();
        Callback::from(move |_| {
            on_click.emit("".to_string())
        })
    };

    html!{
        <select class="player-select">

        <option onclick={default_on_player_select} value=""></option>
        {
            players.iter().map(|player| {
                let on_player_select = {
                    let on_click = on_click.clone();
                    let player = player.clone();
                    Callback::from(move |_| {
                        on_click.emit(player.clone())
                    })
                };

                html!{
                    <option onclick={on_player_select} value={player.clone()}>{player.clone()}</option>
                }
            }
            ).collect::<Html>()
        }

        </select>
    }
}
