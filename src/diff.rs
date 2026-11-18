use similar::{ChangeTag, TextDiff};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DiffStats {
    pub added: usize,
    pub removed: usize,
    pub changed: usize,
    pub unchanged: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SideKind {
    Equal,
    Removed,
    Added,
    Empty,
}

#[derive(Clone, Debug, Default)]
pub struct Segment {
    pub text: String,
    pub emphasis: bool,
}

#[derive(Clone, Debug, Default)]
pub struct DiffSide {
    pub number: Option<usize>,
    pub kind: SideKind,
    pub text: String,
    pub segments: Vec<Segment>,
}

impl Default for SideKind {
    fn default() -> Self {
        Self::Empty
    }
}

#[derive(Clone, Debug, Default)]
pub struct DiffRow {
    pub left: DiffSide,
    pub right: DiffSide,
}

#[derive(Clone, Debug, Default)]
pub struct DiffResult {
    pub rows: Vec<DiffRow>,
    pub stats: DiffStats,
}

impl DiffResult {
    pub fn compute(original: &str, modified: &str) -> Self {
        // Treat CRLF and lone CR as ordinary line endings so a checkout on a
        // different platform does not show every line as changed.
        let original = original.replace("\r\n", "\n").replace('\r', "\n");
        let modified = modified.replace("\r\n", "\n").replace('\r', "\n");
        let diff = TextDiff::from_lines(&original, &modified);
        let mut rows = Vec::new();
        let mut stats = DiffStats::default();

        for op in diff.ops() {
            match op {
                similar::DiffOp::Equal { .. } => {
                    for change in diff.iter_changes(op) {
                        let text = strip_line_ending(change.value()).to_string();
                        rows.push(DiffRow {
                            left: DiffSide {
                                number: change.old_index().map(|index| index + 1),
                                kind: SideKind::Equal,
                                text: text.clone(),
                                segments: vec![Segment {
                                    text,
                                    emphasis: false,
                                }],
                            },
                            right: DiffSide {
                                number: change.new_index().map(|index| index + 1),
                                kind: SideKind::Equal,
                                text: strip_line_ending(change.value()).to_string(),
                                segments: vec![Segment {
                                    text: strip_line_ending(change.value()).to_string(),
                                    emphasis: false,
                                }],
                            },
                        });
                        stats.unchanged += 1;
                    }
                }
                similar::DiffOp::Delete { .. } => {
                    for change in diff.iter_changes(op) {
                        let text = strip_line_ending(change.value()).to_string();
                        rows.push(DiffRow {
                            left: DiffSide {
                                number: change.old_index().map(|index| index + 1),
                                kind: SideKind::Removed,
                                segments: vec![Segment {
                                    text: text.clone(),
                                    emphasis: false,
                                }],
                                text,
                            },
                            right: DiffSide::default(),
                        });
                        stats.removed += 1;
                    }
                }
                similar::DiffOp::Insert { .. } => {
                    for change in diff.iter_changes(op) {
                        let text = strip_line_ending(change.value()).to_string();
                        rows.push(DiffRow {
                            left: DiffSide::default(),
                            right: DiffSide {
                                number: change.new_index().map(|index| index + 1),
                                kind: SideKind::Added,
                                segments: vec![Segment {
                                    text: text.clone(),
                                    emphasis: false,
                                }],
                                text,
                            },
                        });
                        stats.added += 1;
                    }
                }
                similar::DiffOp::Replace { .. } => {
                    let mut deleted = Vec::new();
                    let mut inserted = Vec::new();
                    for change in diff.iter_changes(op) {
                        match change.tag() {
                            ChangeTag::Delete => deleted.push(change),
                            ChangeTag::Insert => inserted.push(change),
                            ChangeTag::Equal => {}
                        }
                    }

                    let paired = deleted.len().min(inserted.len());
                    for index in 0..paired {
                        let old = deleted[index];
                        let new = inserted[index];
                        let old_text = strip_line_ending(old.value()).to_string();
                        let new_text = strip_line_ending(new.value()).to_string();
                        let char_diff = TextDiff::from_chars(&old_text, &new_text);

                        rows.push(DiffRow {
                            left: DiffSide {
                                number: old.old_index().map(|index| index + 1),
                                kind: SideKind::Removed,
                                text: old_text.clone(),
                                segments: segments_for(&char_diff, ChangeTag::Delete, &old_text),
                            },
                            right: DiffSide {
                                number: new.new_index().map(|index| index + 1),
                                kind: SideKind::Added,
                                text: new_text.clone(),
                                segments: segments_for(&char_diff, ChangeTag::Insert, &new_text),
                            },
                        });
                        stats.changed += 1;
                    }

                    for change in &deleted[paired..] {
                        let text = strip_line_ending(change.value()).to_string();
                        rows.push(DiffRow {
                            left: DiffSide {
                                number: change.old_index().map(|index| index + 1),
                                kind: SideKind::Removed,
                                segments: vec![Segment {
                                    text: text.clone(),
                                    emphasis: false,
                                }],
                                text,
                            },
                            right: DiffSide::default(),
                        });
                        stats.removed += 1;
                    }

                    for change in &inserted[paired..] {
                        let text = strip_line_ending(change.value()).to_string();
                        rows.push(DiffRow {
                            left: DiffSide::default(),
                            right: DiffSide {
                                number: change.new_index().map(|index| index + 1),
                                kind: SideKind::Added,
                                segments: vec![Segment {
                                    text: text.clone(),
                                    emphasis: false,
                                }],
                                text,
                            },
                        });
                        stats.added += 1;
                    }
                }
            }
        }

        Self { rows, stats }
    }
}

fn segments_for(
    char_diff: &TextDiff<'_, '_, str>,
    side: ChangeTag,
    expected: &str,
) -> Vec<Segment> {
    let mut segments = Vec::new();
    for change in char_diff.iter_all_changes() {
        let keep = match side {
            ChangeTag::Delete => change.tag() != ChangeTag::Insert,
            ChangeTag::Insert => change.tag() != ChangeTag::Delete,
            ChangeTag::Equal => true,
        };
        if !keep {
            continue;
        }
        let text = change.value().to_string();
        if text.is_empty() {
            continue;
        }
        segments.push(Segment {
            emphasis: change.tag() != ChangeTag::Equal,
            text,
        });
    }

    if segments.iter().map(|segment| segment.text.len()).sum::<usize>() != expected.len() {
        return vec![Segment {
            text: expected.to_string(),
            emphasis: false,
        }];
    }

    segments
}

fn strip_line_ending(line: &str) -> &str {
    line.strip_suffix('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line))
        .unwrap_or(line)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn emphasized(side: &DiffSide) -> String {
        side.segments
            .iter()
            .filter(|segment| segment.emphasis)
            .map(|segment| segment.text.as_str())
            .collect()
    }

    fn joined(side: &DiffSide) -> String {
        side.segments
            .iter()
            .map(|segment| segment.text.as_str())
            .collect()
    }

    #[test]
    fn identical_text_is_unchanged() {
        let diff = DiffResult::compute("alpha\nbravo\n", "alpha\nbravo\n");
        assert_eq!(diff.rows.len(), 2);
        assert_eq!(diff.stats.unchanged, 2);
        assert_eq!(diff.stats.added, 0);
        assert_eq!(diff.stats.removed, 0);
        assert_eq!(diff.stats.changed, 0);
        assert_eq!(diff.rows[0].left.number, Some(1));
        assert_eq!(diff.rows[1].right.number, Some(2));
    }

    #[test]
    fn insertion_and_deletion_keep_other_side_empty() {
        let diff = DiffResult::compute("alpha\nbravo\n", "alpha\nnew\nbravo\n");
        assert_eq!(diff.stats.added, 1);
        assert_eq!(diff.stats.unchanged, 2);
        assert_eq!(diff.rows[1].left.kind, SideKind::Empty);
        assert_eq!(diff.rows[1].right.kind, SideKind::Added);
        assert_eq!(diff.rows[1].right.text, "new");

        let diff = DiffResult::compute("alpha\nnew\nbravo\n", "alpha\nbravo\n");
        assert_eq!(diff.stats.removed, 1);
        assert_eq!(diff.rows[1].left.kind, SideKind::Removed);
        assert_eq!(diff.rows[1].right.kind, SideKind::Empty);
    }

    #[test]
    fn replacement_pairs_lines_and_marks_changed_characters() {
        let diff = DiffResult::compute("hello world\n", "hello brave world\n");
        assert_eq!(diff.stats.changed, 1);
        assert_eq!(diff.rows.len(), 1);
        assert_eq!(diff.rows[0].left.kind, SideKind::Removed);
        assert_eq!(diff.rows[0].right.kind, SideKind::Added);
        assert!(emphasized(&diff.rows[0].right).contains("brave"));
        assert_eq!(diff.rows[0].right.text, "hello brave world");
    }

    #[test]
    fn single_character_change_highlights_only_that_character() {
        let diff = DiffResult::compute("hello\n", "hallo\n");
        assert_eq!(diff.stats.changed, 1);
        assert_eq!(diff.rows.len(), 1);
        assert_eq!(emphasized(&diff.rows[0].left), "e");
        assert_eq!(emphasized(&diff.rows[0].right), "a");
        assert_eq!(diff.rows[0].left.text, "hello");
        assert_eq!(diff.rows[0].right.text, "hallo");
    }

    #[test]
    fn utf8_and_emoji_changes_do_not_split_characters() {
        let diff = DiffResult::compute("café 😀 there\n", "cafè 😃 there\n");
        assert_eq!(diff.stats.changed, 1);
        assert_eq!(emphasized(&diff.rows[0].left), "é😀");
        assert_eq!(emphasized(&diff.rows[0].right), "è😃");
        assert_eq!(joined(&diff.rows[0].left), diff.rows[0].left.text);
        assert_eq!(joined(&diff.rows[0].right), diff.rows[0].right.text);
    }

    #[test]
    fn crlf_and_lf_compare_equal() {
        let diff = DiffResult::compute("alpha\r\nbravo\r\n", "alpha\nbravo\n");
        assert_eq!(diff.stats.unchanged, 2);
        assert_eq!(diff.stats.changed, 0);
    }
}
