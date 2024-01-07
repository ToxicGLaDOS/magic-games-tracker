use implicit_clone::unsync::IArray;
use std::rc::Rc;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct PlayersSelectProps {
    pub players: IArray<Rc<str>>,
    pub on_click: Callback<String>
}

#[function_component(PlayersSelect)]
pub fn players_select(PlayersSelectProps { players, on_click }: &PlayersSelectProps) -> Html {

    let on_click = move |player: Rc<str>| {
        let on_click = on_click.clone();
        Callback::from(move |_| {
            let player = player.clone();
            on_click.emit(player.to_string());
        })
    };

    html!{
        <select class="player-select">

        <option onclick={on_click(Rc::from(""))} value=""></option>
        {
            players.iter().map(|player| {

                html!{
                    <option key={player.to_string()} onclick={on_click(player.clone())} value={player.clone()}>{player.clone()}</option>
                }
            }
            ).collect::<Html>()
        }

        </select>
    }
}
