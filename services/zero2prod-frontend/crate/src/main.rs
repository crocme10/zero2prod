pub mod app;
pub mod components;
pub mod pages;

fn main() {
    yew::Renderer::<app::Main>::new().render();
}
