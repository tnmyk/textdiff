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
        ui.vertical(|ui| {
            ui.label(eframe::egui::RichText::new(label).strong().size(16.0));
            ui.add(eframe::egui::TextEdit::multiline(&mut self.text)
                .desired_width(f32::INFINITY)
                .desired_rows(10)
                .hint_text(format!("Paste {} here...", label))
            );
        });
    }
}
