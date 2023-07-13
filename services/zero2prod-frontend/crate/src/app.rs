use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::backend::Backend;
use crate::components::subscription::Subscription;
use crate::pages::home::Home;
use crate::pages::page_not_found::PageNotFound;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/api/*path")]
    Backend { path: String },
    #[at("/subscription")]
    Subscription,
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
        Route::Backend { path } => {
            html! { <Backend path={path.clone()}/> }
        }
        Route::Subscription => {
            html! { <Subscription /> }
        }
        Route::NotFound => {
            html! { <PageNotFound/> }
        }
    }
}

#[function_component(Main)]
pub fn app() -> Html {
    wasm_logger::init(wasm_logger::Config::default());
    html! {
        <div class="container max-w-full mx-auto md:py-24 px-6">
            <div class="max-w-sm mx-auto px-6">
                <div class="relative flex flex-wrap">
                    <BrowserRouter>
                        <Switch<Route> render={switch} />
                    </BrowserRouter>
                </div>
            </div>
        </div>
    }
}
