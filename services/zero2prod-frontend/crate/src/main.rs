pub mod app;
pub mod pages;

fn main() {
    yew::Renderer::<app::Main>::new().render();
}
