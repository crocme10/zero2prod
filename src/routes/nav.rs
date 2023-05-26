use leptos::{component, view, IntoView, Scope};
use leptos_router::*;

#[component]
pub fn Nav(cx: Scope) -> impl IntoView {
    view! { cx,
        <header>
            <nav class="flex w-full items-center justify-between py-2">
                <A href="/" class="">"Home"</A>
            </nav>
        </header>
    }
}
