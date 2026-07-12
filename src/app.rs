use eframe::egui::{self};

use crate::ui::{compare_results::CompareResults, input::Input};

#[derive(Default)]
pub struct TextDiffApp {
    pub input1: Input,
    pub input2: Input,
    pub result: CompareResults,
}

impl TextDiffApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            input1: Input::new(),
            input2: Input::new(),
            result: CompareResults::new(eframe::egui::text::LayoutJob::default()),
        }
    }
}

impl eframe::App for TextDiffApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            crate::ui::header::render(ui);
            ui.add_space(16.0);
            
            ui.vertical_centered(|ui| {
                ui.columns(2, |cols| {
                    self.input1.render(&mut cols[0], "Original Text");
                    self.input2.render(&mut cols[1], "Modified Text");
                });

                ui.add_space(20.0);
                let compare_btn = ui.add_sized(
                    [200.0, 40.0],
                    egui::Button::new(egui::RichText::new("Compare").size(18.0).strong()),
                );
                
                if compare_btn.clicked() {
                    self.result.compare(&self.input1.text, &self.input2.text, ui);
               }

                self.result.render(ui);
            });
        });
    }
}
