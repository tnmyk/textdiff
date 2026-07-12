use eframe::egui::{self};

use crate::ui::input::Input;

#[derive(Default)]
pub struct TextDiffApp {
    input1: Input,
    input2: Input,
}

impl TextDiffApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            input1: Input::new(),
            input2: Input::new(),
        }
    }
}

impl eframe::App for TextDiffApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.vertical_centered(|ui| {
                crate::ui::header::render(ui);

                self.input1.render(ui, "Input 1");
                self.input2.render(ui, "Input 2");

                if ui.button("Compare").clicked() {
                    let text1 = &self.input1.text;
                    let text2 = &self.input2.text;

                    // TODO: Implement the logic to compare the two inputs and display the differences.
                    println!("Input 1: {}", text1);
                    println!("Input 2: {}", text2);
                }

                ui.label(&self.input1.text);
                ui.label(&self.input2.text);
            });
        });
    }
}
