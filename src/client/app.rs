use yew::prelude::*;

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <main>
            <form>
                <table>
                    <tr>
                        <td><label>{ "Players" }</label></td>
                        <td><select id="player1-select" class="player-select"/></td>
                        <td><select id="player2-select" class="player-select"/></td>
                        <td><select id="player3-select" class="player-select"/></td>
                        <td><select id="player4-select" class="player-select"/></td>
                    </tr>
                    <tr>
                        <label>{ "Commanders" }</label>
                        <td><input id="commander1-input" class="commander-input"/></td>
                        <td><input id="commander2-input" class="commander-input"/></td>
                        <td><input id="commander3-input" class="commander-input"/></td>
                        <td><input id="commander4-input" class="commander-input"/></td>
                    </tr>
                    <tr>
                        <label>{ "Winner" }</label>
                        <td><select id="winner-select" class="winner-select"></select></td>
                    </tr>
                </table>
                    <input type="submit" value="Submit"/>
            </form>
            <br/>
            <form>
                <label>{"Add new player"}</label>
                <input id="new-player-input" class="new-player-input"/>
                <input type="submit" value="Submit"/>
            </form>
        </main>
    }
}
