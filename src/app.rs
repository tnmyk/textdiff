use eframe::egui::{self, Button, CentralPanel, RichText};

use crate::ui::{
    compare_results::CompareResults,
    example::{ExampleAction, render_example},
    input::Input,
    theme,
};

#[derive(PartialEq, Default)]
pub enum AppMode {
    #[default]
    Edit,
    Diff,
}

pub struct TextDiffApp {
    mode: AppMode,
    input1: Input,
    input2: Input,
    result: CompareResults,
}

impl TextDiffApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::apply_theme(&cc.egui_ctx);
        Self {
            mode: AppMode::Edit,
            input1: Input::default(),
            input2: Input::default(),
            result: CompareResults::default(),
        }
    }
}

impl eframe::App for TextDiffApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::bottom("footer").show(ui, |ui| {
            ui.add_space(8.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new("🔒 All data is processed entirely in your browser and never leaves your device.")
                        .size(12.0)
                        .color(egui::Color32::from_rgb(120, 120, 120)),
                );
            });
            ui.add_space(1.0);
        });

        CentralPanel::default().show(ui, |ui| {
            crate::ui::header::render(ui);
            ui.add_space(16.0);

            ui.vertical_centered(|ui| {
                if self.mode == AppMode::Edit {
                    ui.columns(2, |cols| {
                        self.input1.render(&mut cols[0], "Original Text");
                        self.input2.render(&mut cols[1], "Modified Text");
                    });

                    ui.add_space(20.0);

                    ui.horizontal(|ui| {
                        let group_width = 250.0;
                        ui.add_space((ui.available_width() - group_width) / 2.0);
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                            let compare_btn = ui.add_sized(
                                [120.0, 32.0],
                                Button::new(RichText::new("Compare").size(15.0).strong()),
                            );

                            if compare_btn.clicked() {
                                // Extract color before mutably borrowing self.result.
                                let text_color = ui.visuals().text_color();
                                self.result.compare(&self.input1.text, &self.input2.text, text_color);
                                self.mode = AppMode::Diff;
                            }

                            ui.add_space(10.0);

                            match render_example(ui) {
                                ExampleAction::LoadExample => {
                                    self.input1.text = "This is an example of the original text.\nIt has multiple lines.\nThis line will be deleted.".to_string();
                                    self.input2.text = "This is an example of the modified text.\nIt has multiple lines.\nThis line will be added.".to_string();
                                }
                                ExampleAction::None => {}
                            }
                        });
                    });
                } else {
                    // Diff Mode
                    ui.horizontal(|ui| {
                        let back_btn = ui.add_sized(
                            [120.0, 32.0],
                            Button::new(RichText::new("Back to Edit").size(15.0).strong()),
                        );
                        if back_btn.clicked() {
                            self.mode = AppMode::Edit;
                        }
                    });
                    ui.add_space(10.0);
                    self.result.render(ui);
                }
            });
        });
    }
}
