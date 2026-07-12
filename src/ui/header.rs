use eframe::egui::{self, Align, Layout, OpenUrl, RichText};

pub fn render(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("Diff")
                .heading()
                .size(16.0)
                .strong()
                .color(egui::Color32::WHITE),
        );

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui.link("Github").clicked() {
                ui.ctx().open_url(OpenUrl::new_tab(
                    "https://github.com/tanmaykachroo/textdiff",
                ));
            }
        });
    });
    ui.separator();
}
