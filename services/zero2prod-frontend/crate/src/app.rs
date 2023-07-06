use yew::prelude::*;
use yew_router::prelude::*;

use crate::pages::page_not_found::PageNotFound;
use crate::pages::home::Home;

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
    #[not_found]
    #[at("/404")]
    NotFound,
}

#[allow(clippy::let_unit_value)]
fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => {
            html! { <Home/> }
        }
        Route::NotFound => {
            html! { <PageNotFound/> }
        }
    }
}

#[function_component(Main)]
pub fn app() -> Html {
    //wasm_logger::init(wasm_logger::Config::default());
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}
