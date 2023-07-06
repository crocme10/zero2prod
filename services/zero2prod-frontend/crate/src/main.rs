use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    html! {
        <h1 class="font-text">{ "Hello World" }</h1>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
