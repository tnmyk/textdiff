use eframe::egui::{self, Button, CentralPanel, RichText};

use crate::ui::{compare_results::CompareResults, example::render_example, input::Input};

#[derive(PartialEq)]
pub enum AppMode {
    Edit,
    Diff,
}

impl Default for AppMode {
    fn default() -> Self {
        Self::Edit
    }
}

#[derive(Default)]
pub struct TextDiffApp {
    pub mode: AppMode,
    pub input1: Input,
    pub input2: Input,
    pub result: CompareResults,
}

impl TextDiffApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            mode: AppMode::Edit,
            input1: Input::new(),
            input2: Input::new(),
            result: CompareResults::default(),
        }
    }
}

impl eframe::App for TextDiffApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        crate::ui::theme::apply_theme(ui.ctx());
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
                        let estimated_total_width = 330.0;
                        ui.add_space((ui.available_width() - estimated_total_width) / 2.0);

                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                            let compare_btn = ui.add_sized(
                                [200.0, 40.0],
                                Button::new(RichText::new("Compare").size(18.0).strong()),
                            );

                            if compare_btn.clicked() {
                                self.result.compare(&self.input1.text, &self.input2.text, ui);
                                self.mode = AppMode::Diff;
                            }

                            ui.add_space(10.0);

                            render_example(self, ui);
                        });
                    });
                } else {
                    // Diff Mode
                    ui.horizontal(|ui| {
                        let back_btn = ui.add_sized(
                            [200.0, 40.0],
                            Button::new(RichText::new("Back to Edit").size(18.0).strong()),
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
