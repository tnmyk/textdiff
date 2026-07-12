use eframe::egui;

pub enum ExampleAction {
    LoadExample,
    None,
}

pub fn render_example(ui: &mut egui::Ui) -> ExampleAction {
    if ui.link("Load example text").clicked() {
        ExampleAction::LoadExample
    } else {
        ExampleAction::None
    }
}
