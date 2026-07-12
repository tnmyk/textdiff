use eframe::egui::{RichText, TextEdit, ScrollArea};

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
            ui.label(RichText::new(label).strong().size(16.0));
            ScrollArea::vertical()
                .id_salt(format!("scroll_{}", label))
                .max_height(350.0)
                .show(ui, |ui| {
                    ui.add(
                        TextEdit::multiline(&mut self.text)
                            .desired_width(f32::INFINITY)
                            .desired_rows(20)
                            .hint_text(format!("Paste {} here...", label)),
                    );
                });
        });
    }
}
