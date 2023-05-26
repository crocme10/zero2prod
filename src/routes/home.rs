use leptos::*;

#[component]
pub fn Home(cx: Scope) -> impl IntoView {
    view! { cx,
        <div class="my-0 mx-auto max-w-3xl text-center font-FiraSans">
            <h2 class="p-6 text-2xl">"Welcome to Leptos with Tailwind"</h2>
        </div>
    }
}
