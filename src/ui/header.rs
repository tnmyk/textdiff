use eframe::egui::{self, Align, Layout, OpenUrl, RichText, vec2};

pub fn render(ui: &mut egui::Ui) {
    ui.add_space(6.0);
    let height = 16.0;
    ui.allocate_ui_with_layout(
        vec2(ui.available_width(), height),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.add_space(8.0);
            ui.label(
                RichText::new("Diff")
                    .heading()
                    .size(12.0)
                    .strong()
                    .color(egui::Color32::WHITE),
            );

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add_space(8.0);
                if ui.link("Github").clicked() {
                    ui.ctx().open_url(OpenUrl::new_tab(
                        "https://github.com/tanmaykachroo/textdiff",
                    ));
                }
            });
        },  
    );
    ui.separator();
}
