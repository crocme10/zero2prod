use yew::prelude::*;

#[function_component(PageNotFound)]
pub fn home() -> Html {
    html! {
        <div class={classes!("flex", "h-screen", "antialiased", "text-slate-900", "dark:text-white")}>
            <div class={classes!("container", "mx-auto", "flex", "flex-col", "items-center", "py-12", "sm:py-24")}>
                <div class={classes!("w-11/12", "sm:w-2/3", "lg:flex", "justify-center", "items-center", "flex-col", "", "mb-5", "sm:mb-10")}>
                    <h1 class={classes!("font-text", "text-5xl")}>{"Not Found"}</h1>
                </div>
            </div>
        </div>
    }
}
