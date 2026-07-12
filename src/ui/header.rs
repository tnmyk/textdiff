use eframe::egui::{self, Align, Layout, OpenUrl, RichText};

pub fn render(ui: &mut egui::Ui) {
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.label(RichText::new("TextDiff").heading().size(20.0).strong());
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui.link("Github").clicked() {
                ui.ctx().open_url(OpenUrl::new_tab(
                    "https://github.com/tanmaykachroo/textdiff",
                ));
            }
        });
    });
    ui.add_space(8.0);
    ui.separator();
}
