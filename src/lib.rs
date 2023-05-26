use cfg_if::cfg_if;
use leptos::{component, view, IntoView, Scope};
use leptos_meta::*;
use leptos_router::*;
// mod api;
pub mod error_template;
pub mod fallback;
pub mod handlers;
mod routes;
use routes::{home::*, nav::*};

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    provide_meta_context(cx);
    view! {
        cx,
        <>
            <Link rel="shortcut icon" type_="image/ico" href="/favicon.ico"/>
            <Stylesheet id="leptos" href="/pkg/zero_to_prod.css"/>
            <Meta name="description" content="Zero to Prod demo."/>
            <Router>
                <Nav />
                <main>
                    <Routes>
                        <Route path="" view=|cx| view! { cx,  <Home/> }/>
                    </Routes>
                </main>
            </Router>
        </>
    }
}

// Needs to be in lib.rs AFAIK because wasm-bindgen needs us to be compiling a lib. I may be wrong.
cfg_if! {
    if #[cfg(feature = "hydrate")] {
        use wasm_bindgen::prelude::wasm_bindgen;

        #[wasm_bindgen]
        pub fn hydrate() {
            _ = console_log::init_with_level(log::Level::Debug);
            console_error_panic_hook::set_once();
            leptos::mount_to_body(move |cx| {
                view! { cx, <App/> }
            });
        }
    }
}
