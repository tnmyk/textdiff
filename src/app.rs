use eframe::egui::{self};

use crate::ui::input::Input;

#[derive(Default)]
pub struct TextDiffApp {
    input1: Input,
    input2: Input,
    result: Option<String>,
}

impl TextDiffApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            input1: Input::new(),
            input2: Input::new(),
            result: None,
        }
    }
}

impl eframe::App for TextDiffApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.vertical_centered(|ui| {
                crate::ui::header::render(ui);
                
                ui.horizontal(|ui| {
                    self.input1.render(ui, "Input 1");
                    self.input2.render(ui, "Input 2");
                });

                if ui.button("Compare").clicked() {
                    let text1 = &self.input1.text;
                    let text2 = &self.input2.text;
                    
                    let mut diff_output = String::new();

                    similar::TextDiff::from_lines(text1, text2)
                        .iter_all_changes()
                        .for_each(|change| {
                            let line = match change.tag() {
                                similar::ChangeTag::Delete => {
                                    format!("- {}", change)
                                }
                                similar::ChangeTag::Insert => {
                                    format!("+ {}", change)
                                }
                                similar::ChangeTag::Equal => {
                                    format!("  {}", change)
                                }
                            };
                            diff_output.push_str(&line);
                            diff_output.push('\n'); // Add a newline for readability
                        });
                    self.result = Some(diff_output);
                }


                if let Some(ref mut result_text) = self.result {
                    ui.add_space(10.0);
                    ui.label("Differences:");
                    ui.text_edit_multiline(result_text);
                }

                ui.label(&self.input1.text);
                ui.label(&self.input2.text);
            });
        });
    }
}
