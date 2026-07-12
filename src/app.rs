use eframe::egui::{self};

use crate::ui::input::Input;

#[derive(Default)]
pub struct TextDiffApp {
    input1: Input,
    input2: Input,
    result: Option<Vec<(similar::ChangeTag, String)>>,
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
                    let text1 = &self.input1.text;
                    let text2 = &self.input2.text;
                    
                    let mut changes = Vec::new();
                    similar::TextDiff::from_lines(text1, text2)
                        .iter_all_changes()
                        .for_each(|change| {
                            changes.push((change.tag(), change.to_string()));
                        });
                    self.result = Some(changes);
                }

                if let Some(ref changes) = self.result {
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new("Differences:").heading());
                    ui.add_space(5.0);
                    
                    egui::Frame::new()
                        .fill(ui.visuals().extreme_bg_color)
                        .corner_radius(8.0)
                        .inner_margin(12.0)
                        .show(ui, |ui| {
                            egui::ScrollArea::vertical()
                                .max_height(400.0)
                                .auto_shrink([false; 2])
                                .show(ui, |ui| {
                                    ui.horizontal_wrapped(|ui| {
                                        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
                                        for (tag, text) in changes {
                                            let (color, bg_color) = match tag {
                                                similar::ChangeTag::Delete => (egui::Color32::WHITE, egui::Color32::from_rgb(130, 0, 0)),
                                                similar::ChangeTag::Insert => (egui::Color32::BLACK, egui::Color32::from_rgb(0, 150, 0)),
                                                similar::ChangeTag::Equal => (ui.visuals().text_color(), egui::Color32::TRANSPARENT),
                                            };
                                            
                                            ui.label(egui::RichText::new(text)
                                                .color(color)
                                                .background_color(bg_color)
                                                .family(egui::FontFamily::Monospace)
                                                .size(14.0)
                                            );
                                        }
                                    });
                                });
                        });
                }
            });
        });
    }
}
