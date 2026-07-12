use crate::app::TextDiffApp;

pub fn render_example(state: &mut TextDiffApp, ui: &mut eframe::egui::Ui) {
    if ui.link("Load example text").clicked() {
        state.input1.text = "This is an example of the original text.\nIt has multiple lines.\nThis line will be deleted.".to_string();
        state.input2.text = "This is an example of the modified text.\nIt has multiple lines.\nThis line will be added.".to_string();
    }
}
