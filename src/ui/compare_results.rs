use eframe::egui::{
    self, Color32, FontId, Frame, RichText, ScrollArea,
    text::{self, LayoutJob},
};

pub struct CompareResults {
    pub job_left: Option<LayoutJob>,
    pub job_right: Option<LayoutJob>,
    pub scroll_offset: f32,
}

impl Default for CompareResults {
    fn default() -> Self {
        Self {
            job_left: None,
            job_right: None,
            scroll_offset: 0.0,
        }
    }
}

impl CompareResults {
    pub fn render(&mut self, ui: &mut egui::Ui) {
        if let (Some(job_left), Some(job_right)) = (&self.job_left, &self.job_right) {
            ui.add_space(10.0);
            ui.label(RichText::new("Side-by-Side Differences:").heading());
            ui.add_space(5.0);

            Frame::new()
                .fill(ui.visuals().extreme_bg_color)
                .corner_radius(8.0)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    let inner_height = ui.available_height();
                    let title_color = ui.visuals().text_color();
                    ui.columns(2, |cols| {
                        // Left Column (Original)
                        cols[0].label(RichText::new("Original Text").strong().color(title_color));
                        cols[0].add_space(4.0);
                        let left_resp = ScrollArea::vertical()
                            .id_salt("left_scroll")
                            .max_height(inner_height - 24.0)
                            .vertical_scroll_offset(self.scroll_offset)
                            .show(&mut cols[0], |ui| {
                                ui.label(job_left.clone());
                            });

                        // Right Column (Modified)
                        cols[1].label(RichText::new("Modified Text").strong().color(title_color));
                        cols[1].add_space(4.0);
                        let right_resp = ScrollArea::vertical()
                            .id_salt("right_scroll")
                            .max_height(inner_height - 24.0)
                            .vertical_scroll_offset(self.scroll_offset)
                            .show(&mut cols[1], |ui| {
                                ui.label(job_right.clone());
                            });

                        // Synchronize scrolling
                        if left_resp.state.offset.y != self.scroll_offset {
                            self.scroll_offset = left_resp.state.offset.y;
                        } else if right_resp.state.offset.y != self.scroll_offset {
                            self.scroll_offset = right_resp.state.offset.y;
                        }
                    });
                });
        }
    }

    pub fn compare(&mut self, text1: &str, text2: &str, ui: &mut egui::Ui) {
        self.job_left = None;
        self.job_right = None;
        self.scroll_offset = 0.0;

        let mut job_left = text::LayoutJob::default();
        let mut job_right = text::LayoutJob::default();
        let diff = similar::TextDiff::from_lines(text1, text2);

        let text_color = ui.visuals().text_color();
        let light_red = Color32::from_rgb(72, 42, 43); // #482A2B
        let dark_red = Color32::from_rgb(120, 29, 32); // #781D20
        let light_green = Color32::from_rgb(41, 68, 54); // #294436
        let dark_green = Color32::from_rgb(31, 97, 62); // #1F613E
        let transparent = Color32::TRANSPARENT;

        job_left.wrap = text::TextWrapping {
            max_width: f32::INFINITY,
            ..Default::default()
        };
        job_right.wrap = text::TextWrapping {
            max_width: f32::INFINITY,
            ..Default::default()
        };

        for op in diff.ops() {
            match op {
                similar::DiffOp::Replace { .. } => {
                    let mut deleted_lines = Vec::new();
                    let mut inserted_lines = Vec::new();
                    let mut deleted_str = String::new();
                    let mut inserted_str = String::new();

                    for change in diff.iter_changes(op) {
                        if change.tag() == similar::ChangeTag::Delete {
                            deleted_lines.push(change.value());
                            deleted_str.push_str(change.value());
                        } else if change.tag() == similar::ChangeTag::Insert {
                            inserted_lines.push(change.value());
                            inserted_str.push_str(change.value());
                        }
                    }

                    let word_diff = similar::TextDiff::from_words(&deleted_str, &inserted_str);

                    // Render left side (Deletions)
                    for change in word_diff.iter_all_changes() {
                        match change.tag() {
                            similar::ChangeTag::Delete => {
                                job_left.append(
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
                                job_left.append(
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

                    // Render right side (Insertions)
                    for change in word_diff.iter_all_changes() {
                        match change.tag() {
                            similar::ChangeTag::Insert => {
                                job_right.append(
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
                                job_right.append(
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

                    // Pad missing lines to keep vertical sync
                    let del_count = deleted_lines.len();
                    let ins_count = inserted_lines.len();

                    if del_count < ins_count {
                        for _ in 0..(ins_count - del_count) {
                            job_left.append(
                                "\n",
                                0.0,
                                text::TextFormat {
                                    font_id: FontId::monospace(14.0),
                                    color: transparent,
                                    background: transparent,
                                    ..Default::default()
                                },
                            );
                        }
                    } else if ins_count < del_count {
                        for _ in 0..(del_count - ins_count) {
                            job_right.append(
                                "\n",
                                0.0,
                                text::TextFormat {
                                    font_id: FontId::monospace(14.0),
                                    color: transparent,
                                    background: transparent,
                                    ..Default::default()
                                },
                            );
                        }
                    }
                }
                similar::DiffOp::Delete { .. } => {
                    let mut lines_count = 0;
                    for change in diff.iter_changes(op) {
                        lines_count += 1;
                        job_left.append(
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
                    for _ in 0..lines_count {
                        job_right.append(
                            "\n",
                            0.0,
                            text::TextFormat {
                                font_id: FontId::monospace(14.0),
                                color: transparent,
                                background: transparent,
                                ..Default::default()
                            },
                        );
                    }
                }
                similar::DiffOp::Insert { .. } => {
                    let mut lines_count = 0;
                    for change in diff.iter_changes(op) {
                        lines_count += 1;
                        job_right.append(
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
                    for _ in 0..lines_count {
                        job_left.append(
                            "\n",
                            0.0,
                            text::TextFormat {
                                font_id: FontId::monospace(14.0),
                                color: transparent,
                                background: transparent,
                                ..Default::default()
                            },
                        );
                    }
                }
                similar::DiffOp::Equal { .. } => {
                    for change in diff.iter_changes(op) {
                        job_left.append(
                            change.value(),
                            0.0,
                            text::TextFormat {
                                font_id: FontId::monospace(14.0),
                                color: text_color,
                                background: transparent,
                                ..Default::default()
                            },
                        );
                        job_right.append(
                            change.value(),
                            0.0,
                            text::TextFormat {
                                font_id: FontId::monospace(14.0),
                                color: text_color,
                                background: transparent,
                                ..Default::default()
                            },
                        );
                    }
                }
            }
        }

        self.job_left = Some(job_left);
        self.job_right = Some(job_right);
    }
}
