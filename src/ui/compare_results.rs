use eframe::egui::{self, Color32, FontId, Frame, RichText, ScrollArea, text::{self, LayoutJob}};
#[derive(Default)]

pub struct CompareResults {
    pub job: Option<LayoutJob>,
}

impl CompareResults {
    pub fn new(job: LayoutJob) -> Self {
        Self { job: Some(job) }
    }

    pub fn render(&self, ui: &mut egui::Ui) {
        if let Some(ref job) = self.job {
            ui.add_space(10.0);
            ui.label(RichText::new("Differences:").heading());
            ui.add_space(5.0);

            Frame::new()
                .fill(ui.visuals().extreme_bg_color)
                .corner_radius(8.0)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ScrollArea::vertical()
                        .max_height(400.0)
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            ui.label(job.clone());
                        });
                });
        }
    }

    pub fn compare(&mut self, text1: &str, text2: &str, ui: &mut egui::Ui) {
        self.job = None;

        let mut job = text::LayoutJob::default();
        let diff = similar::TextDiff::from_lines(text1, text2);

        let text_color = ui.visuals().text_color();
        let light_red = Color32::from_rgb(60, 20, 20);
        let dark_red = Color32::from_rgb(180, 50, 50);
        let light_green = Color32::from_rgb(20, 60, 20);
        let dark_green = Color32::from_rgb(50, 180, 50);

        for op in diff.ops() {
            match op {
                similar::DiffOp::Replace { .. } => {
                    let mut deleted = String::new();
                    let mut inserted = String::new();
                    for change in diff.iter_changes(op) {
                        if change.tag() == similar::ChangeTag::Delete {
                            deleted.push_str(change.value());
                        } else if change.tag() == similar::ChangeTag::Insert {
                            inserted.push_str(change.value());
                        }
                    }

                    let word_diff = similar::TextDiff::from_words(&deleted, &inserted);

                    // Render deleted line(s)
                    for change in word_diff.iter_all_changes() {
                        match change.tag() {
                            similar::ChangeTag::Delete => {
                                job.append(
                                    change.value(),
                                    0.0,
                                    text::TextFormat {
                                        font_id: FontId::monospace(14.0),
                                        color: Color32::WHITE,
                                        background: dark_red,
                                        ..Default::default()
                                    },
                                );
                            }
                            similar::ChangeTag::Equal => {
                                job.append(
                                    change.value(),
                                    0.0,
                                    text::TextFormat {
                                        font_id: FontId::monospace(14.0),
                                        color: text_color,
                                        background: light_red,
                                        ..Default::default()
                                    },
                                );
                            }
                            similar::ChangeTag::Insert => {}
                        }
                    }

                    // Render inserted line(s)
                    for change in word_diff.iter_all_changes() {
                        match change.tag() {
                            similar::ChangeTag::Insert => {
                                job.append(
                                    change.value(),
                                    0.0,
                                    text::TextFormat {
                                        font_id: FontId::monospace(14.0),
                                        color: Color32::WHITE,
                                        background: dark_green,
                                        ..Default::default()
                                    },
                                );
                            }
                            similar::ChangeTag::Equal => {
                                job.append(
                                    change.value(),
                                    0.0,
                                    text::TextFormat {
                                        font_id: FontId::monospace(14.0),
                                        color: text_color,
                                        background: light_green,
                                        ..Default::default()
                                    },
                                );
                            }
                            similar::ChangeTag::Delete => {}
                        }
                    }
                }
                _ => {
                    for change in diff.iter_changes(op) {
                        let (color, bg_color) = match change.tag() {
                            similar::ChangeTag::Delete => (text_color, light_red),
                            similar::ChangeTag::Insert => (text_color, light_green),
                            similar::ChangeTag::Equal => (text_color, Color32::TRANSPARENT),
                        };
                        job.append(
                            change.value(),
                            0.0,
                            text::TextFormat {
                                font_id: FontId::monospace(14.0),
                                color,
                                background: bg_color,
                                ..Default::default()
                            },
                        );
                    }
                }
            }
        }

        self.job = Some(job);
    }
}
