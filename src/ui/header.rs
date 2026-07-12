use eframe::egui::{self, RichText};

pub fn render(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("TextDiff").heading().strong());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.link("Github").clicked() {
                ui.ctx().open_url(egui::OpenUrl::new_tab(
                    "https://github.com/tanmaykachroo/textdiff",
                ));
            }
        });
    });
    ui.separator();
}
