#[derive(Default)]
pub struct Input {
    pub text: String,
}

impl Input {
    pub fn new() -> Self {
        Self {
            text: String::new(),
        }
    }

    pub fn render(&mut self, ui: &mut eframe::egui::Ui, label: &str) {
        ui.label(label);
        ui.text_edit_multiline(&mut self.text);
    }
}
