
use imgui::{ Context, * };

mod renderer;
use renderer::{ AppRenderer, Fonts, FontStyle };

mod application;
use application::Application;

fn main() {
    let renderer = AppRenderer::new();
    let mut application = Application::new(renderer.fonts().clone());
    renderer.run(move |ui| { application.build_gui(ui) });
}
