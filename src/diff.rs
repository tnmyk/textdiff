use similar::{ChangeTag, TextDiff};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CompareOptions {
    pub ignore_case: bool,
    pub ignore_whitespace: bool,
    pub ignore_blank_lines: bool,
}

impl CompareOptions {
    pub fn active_labels(&self) -> Vec<&'static str> {
        let mut labels = Vec::new();
        if self.ignore_case {
            labels.push("case");
        }
        if self.ignore_whitespace {
            labels.push("whitespace");
        }
        if self.ignore_blank_lines {
            labels.push("blank lines");
        }
        labels
    }
}

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

impl DiffRow {
    /// True when either side of the row has an added or removed line.
    pub fn is_change(&self) -> bool {
        matches!(self.left.kind, SideKind::Removed | SideKind::Added)
            || matches!(self.right.kind, SideKind::Removed | SideKind::Added)
    }
}

/// A contiguous run of changed rows.
///
/// `start` is inclusive and `end` is exclusive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChangeHunk {
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug, Default)]
pub struct DiffResult {
    pub rows: Vec<DiffRow>,
    pub stats: DiffStats,
}

impl DiffResult {
    /// Group contiguous changed rows into hunks for navigation and collapsing.
    pub fn change_hunks(&self) -> Vec<ChangeHunk> {
        let mut hunks = Vec::new();
        let mut start = None;

        for (index, row) in self.rows.iter().enumerate() {
            if row.is_change() {
                start.get_or_insert(index);
            } else if let Some(start) = start.take() {
                hunks.push(ChangeHunk { start, end: index });
            }
        }

        if let Some(start) = start {
            hunks.push(ChangeHunk {
                start,
                end: self.rows.len(),
            });
        }

        hunks
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn compute(original: &str, modified: &str) -> Self {
        Self::compute_with_options(original, modified, CompareOptions::default())
    }

    pub fn compute_with_options(original: &str, modified: &str, options: CompareOptions) -> Self {
        // Treat CRLF and lone CR as ordinary line endings so a checkout on a
        // different platform does not show every line as changed.
        let original = original.replace("\r\n", "\n").replace('\r', "\n");
        let modified = modified.replace("\r\n", "\n").replace('\r', "\n");
        let old_meta = line_meta(&original, options);
        let new_meta = line_meta(&modified, options);

        // The comparison sequence contains only the lines the options ask us to
        // compare. Keeping a separate map back to the metadata means every row
        // still shows the user's original text and line number.
        let old_seq: Vec<usize> = (0..old_meta.len())
            .filter(|&index| old_meta[index].compared)
            .collect();
        let new_seq: Vec<usize> = (0..new_meta.len())
            .filter(|&index| new_meta[index].compared)
            .collect();
        let old_keys: Vec<&str> = old_seq
            .iter()
            .map(|&index| old_meta[index].key.as_str())
            .collect();
        let new_keys: Vec<&str> = new_seq
            .iter()
            .map(|&index| new_meta[index].key.as_str())
            .collect();

        let diff = TextDiff::from_slices(&old_keys, &new_keys);
        let mut specs = Vec::new();
        let mut stats = DiffStats::default();

        for op in diff.ops() {
            match op {
                similar::DiffOp::Equal { .. } => {
                    for change in diff.iter_changes(op) {
                        if let (Some(old), Some(new)) = (change.old_index(), change.new_index()) {
                            specs.push(equal_spec(
                                old_seq[old],
                                new_seq[new],
                                &old_meta,
                                &new_meta,
                            ));
                            stats.unchanged += 1;
                        }
                    }
                }
                similar::DiffOp::Delete { .. } => {
                    for change in diff.iter_changes(op) {
                        if let Some(index) = change.old_index() {
                            specs.push(removed_spec(old_seq[index], &old_meta));
                            stats.removed += 1;
                        }
                    }
                }
                similar::DiffOp::Insert { .. } => {
                    for change in diff.iter_changes(op) {
                        if let Some(index) = change.new_index() {
                            specs.push(added_spec(new_seq[index], &new_meta));
                            stats.added += 1;
                        }
                    }
                }
                similar::DiffOp::Replace { .. } => {
                    let mut deleted = Vec::new();
                    let mut inserted = Vec::new();
                    for change in diff.iter_changes(op) {
                        match change.tag() {
                            ChangeTag::Delete => {
                                if let Some(index) = change.old_index() {
                                    deleted.push(old_seq[index]);
                                }
                            }
                            ChangeTag::Insert => {
                                if let Some(index) = change.new_index() {
                                    inserted.push(new_seq[index]);
                                }
                            }
                            ChangeTag::Equal => {}
                        }
                    }

                    let paired = deleted.len().min(inserted.len());
                    for index in 0..paired {
                        let old = &old_meta[deleted[index]];
                        let new = &new_meta[inserted[index]];
                        let char_diff =
                            TextDiff::configure().diff_graphemes(&old.display, &new.display);

                        specs.push(RowSpec {
                            old: Some(deleted[index]),
                            new: Some(inserted[index]),
                            left_kind: SideKind::Removed,
                            right_kind: SideKind::Added,
                            left_segments: segments_for(
                                &char_diff,
                                ChangeTag::Delete,
                                &old.display,
                            ),
                            right_segments: segments_for(
                                &char_diff,
                                ChangeTag::Insert,
                                &new.display,
                            ),
                        });
                        stats.changed += 1;
                    }

                    for &index in &deleted[paired..] {
                        specs.push(removed_spec(index, &old_meta));
                        stats.removed += 1;
                    }

                    for &index in &inserted[paired..] {
                        specs.push(added_spec(index, &new_meta));
                        stats.added += 1;
                    }
                }
            }
        }

        let rows = build_rows(&specs, &old_meta, &new_meta);
        Self { rows, stats }
    }
}

struct LineMeta {
    number: usize,
    display: String,
    key: String,
    compared: bool,
}

struct RowSpec {
    old: Option<usize>,
    new: Option<usize>,
    left_kind: SideKind,
    right_kind: SideKind,
    left_segments: Vec<Segment>,
    right_segments: Vec<Segment>,
}

fn line_meta(text: &str, options: CompareOptions) -> Vec<LineMeta> {
    split_lines(text)
        .into_iter()
        .enumerate()
        .map(|(index, display)| {
            let key = comparison_key(&display, options);
            let compared = !(options.ignore_blank_lines && key.trim().is_empty());
            LineMeta {
                number: index + 1,
                display,
                key,
                compared,
            }
        })
        .collect()
}

fn split_lines(text: &str) -> Vec<String> {
    if text.is_empty() {
        return Vec::new();
    }
    let mut lines: Vec<String> = text.split('\n').map(|line| line.to_string()).collect();
    if text.ends_with('\n') {
        lines.pop();
    }
    lines
}

/// Comparison rule: `ignore_whitespace` collapses every run of Unicode
/// whitespace to a single space and trims both ends; `ignore_case` lowercases
/// the already whitespace-normalized line.
fn comparison_key(line: &str, options: CompareOptions) -> String {
    let mut key = if options.ignore_whitespace {
        line.split_whitespace().collect::<Vec<_>>().join(" ")
    } else {
        line.to_string()
    };
    if options.ignore_case {
        key = key.to_lowercase();
    }
    key
}

fn equal_spec(old: usize, new: usize, old_meta: &[LineMeta], new_meta: &[LineMeta]) -> RowSpec {
    RowSpec {
        old: Some(old),
        new: Some(new),
        left_kind: SideKind::Equal,
        right_kind: SideKind::Equal,
        left_segments: plain_segments(&old_meta[old].display),
        right_segments: plain_segments(&new_meta[new].display),
    }
}

fn removed_spec(old: usize, old_meta: &[LineMeta]) -> RowSpec {
    RowSpec {
        old: Some(old),
        new: None,
        left_kind: SideKind::Removed,
        right_kind: SideKind::Empty,
        left_segments: plain_segments(&old_meta[old].display),
        right_segments: Vec::new(),
    }
}

fn added_spec(new: usize, new_meta: &[LineMeta]) -> RowSpec {
    RowSpec {
        old: None,
        new: Some(new),
        left_kind: SideKind::Empty,
        right_kind: SideKind::Added,
        left_segments: Vec::new(),
        right_segments: plain_segments(&new_meta[new].display),
    }
}

fn plain_segments(text: &str) -> Vec<Segment> {
    vec![Segment {
        text: text.to_string(),
        emphasis: false,
    }]
}

fn build_rows(specs: &[RowSpec], old_meta: &[LineMeta], new_meta: &[LineMeta]) -> Vec<DiffRow> {
    let mut rows = Vec::with_capacity(specs.len());
    let mut next_old = 0;
    let mut next_new = 0;

    for spec in specs {
        flush_ignored(
            &mut rows,
            old_meta,
            new_meta,
            &mut next_old,
            &mut next_new,
            spec.old,
            spec.new,
        );
        rows.push(DiffRow {
            left: side_for(spec.old, spec.left_kind, &spec.left_segments, old_meta),
            right: side_for(spec.new, spec.right_kind, &spec.right_segments, new_meta),
        });
        if let Some(index) = spec.old {
            next_old = index + 1;
        }
        if let Some(index) = spec.new {
            next_new = index + 1;
        }
    }

    // Blank lines excluded from the comparison still appear as neutral rows so
    // the displayed text and line numbers remain complete.
    flush_ignored(
        &mut rows,
        old_meta,
        new_meta,
        &mut next_old,
        &mut next_new,
        Some(old_meta.len()),
        Some(new_meta.len()),
    );
    rows
}

fn flush_ignored(
    rows: &mut Vec<DiffRow>,
    old_meta: &[LineMeta],
    new_meta: &[LineMeta],
    next_old: &mut usize,
    next_new: &mut usize,
    old_until: Option<usize>,
    new_until: Option<usize>,
) {
    let mut old_pending = Vec::new();
    let mut new_pending = Vec::new();

    if let Some(until) = old_until {
        while *next_old < until {
            let line = &old_meta[*next_old];
            if line.compared {
                break;
            }
            old_pending.push(*next_old);
            *next_old += 1;
        }
    }
    if let Some(until) = new_until {
        while *next_new < until {
            let line = &new_meta[*next_new];
            if line.compared {
                break;
            }
            new_pending.push(*next_new);
            *next_new += 1;
        }
    }

    for index in 0..old_pending.len().max(new_pending.len()) {
        let left = old_pending
            .get(index)
            .map(|&index| neutral_side(&old_meta[index]))
            .unwrap_or_default();
        let right = new_pending
            .get(index)
            .map(|&index| neutral_side(&new_meta[index]))
            .unwrap_or_default();
        rows.push(DiffRow { left, right });
    }
}

fn side_for(
    index: Option<usize>,
    kind: SideKind,
    segments: &[Segment],
    meta: &[LineMeta],
) -> DiffSide {
    match index {
        Some(index) => {
            let line = &meta[index];
            DiffSide {
                number: Some(line.number),
                kind,
                text: line.display.clone(),
                segments: segments.to_vec(),
            }
        }
        None => DiffSide::default(),
    }
}

fn neutral_side(line: &LineMeta) -> DiffSide {
    DiffSide {
        number: Some(line.number),
        kind: SideKind::Equal,
        text: line.display.clone(),
        segments: plain_segments(&line.display),
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

    if segments
        .iter()
        .map(|segment| segment.text.len())
        .sum::<usize>()
        != expected.len()
    {
        return vec![Segment {
            text: expected.to_string(),
            emphasis: false,
        }];
    }

    segments
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
    fn grapheme_clusters_are_not_split() {
        let diff = DiffResult::compute("a👍🏽b\n", "a👍🏻b\n");
        assert_eq!(emphasized(&diff.rows[0].left), "👍🏽");
        assert_eq!(emphasized(&diff.rows[0].right), "👍🏻");
        assert_eq!(joined(&diff.rows[0].left), "a👍🏽b");

        let diff = DiffResult::compute("👨‍👩‍👧‍👦\n", "👨‍👩‍👧\n");
        assert_eq!(emphasized(&diff.rows[0].left), "👨‍👩‍👧‍👦");
        assert_eq!(emphasized(&diff.rows[0].right), "👨‍👩‍👧");

        let diff = DiffResult::compute("e\u{301}\n", "é\n");
        assert_eq!(joined(&diff.rows[0].left), "e\u{301}");
        assert_eq!(emphasized(&diff.rows[0].left), "e\u{301}");
        assert_eq!(emphasized(&diff.rows[0].right), "é");
    }

    #[test]
    fn crlf_and_lf_compare_equal() {
        let diff = DiffResult::compute("alpha\r\nbravo\r\n", "alpha\nbravo\n");
        assert_eq!(diff.stats.unchanged, 2);
        assert_eq!(diff.stats.changed, 0);
    }

    #[test]
    fn ignore_case_treats_case_only_changes_as_equal() {
        let options = CompareOptions {
            ignore_case: true,
            ..Default::default()
        };
        let diff = DiffResult::compute_with_options("Hello World\n", "hello world\n", options);

        assert_eq!(diff.stats.unchanged, 1);
        assert_eq!(diff.stats.changed, 0);
        assert_eq!(diff.rows.len(), 1);
        assert_eq!(diff.rows[0].left.kind, SideKind::Equal);
        assert_eq!(diff.rows[0].right.kind, SideKind::Equal);
        assert_eq!(diff.rows[0].left.text, "Hello World");
        assert_eq!(diff.rows[0].right.text, "hello world");
        assert_eq!(emphasized(&diff.rows[0].left), "");
        assert_eq!(emphasized(&diff.rows[0].right), "");

        let strict = DiffResult::compute("Hello World\n", "hello world\n");
        assert_eq!(strict.stats.changed, 1);
    }

    #[test]
    fn ignore_whitespace_collapses_and_trims() {
        let options = CompareOptions {
            ignore_whitespace: true,
            ignore_case: true,
            ..Default::default()
        };
        let diff =
            DiffResult::compute_with_options("  Hello\t  World  \n", "hello world\n", options);

        assert_eq!(diff.stats.unchanged, 1);
        assert_eq!(diff.stats.changed, 0);
        assert_eq!(diff.rows[0].left.text, "  Hello\t  World  ");
        assert_eq!(diff.rows[0].right.text, "hello world");
        assert_eq!(emphasized(&diff.rows[0].left), "");
    }

    #[test]
    fn ignore_blank_lines_keeps_original_numbers_and_display() {
        let options = CompareOptions {
            ignore_blank_lines: true,
            ..Default::default()
        };
        let diff = DiffResult::compute_with_options("alpha\n\nbravo\n", "alpha\nbravo\n", options);

        assert_eq!(diff.stats.unchanged, 2);
        assert_eq!(diff.stats.added, 0);
        assert_eq!(diff.stats.removed, 0);
        assert_eq!(diff.stats.changed, 0);
        assert_eq!(diff.rows.len(), 3);
        assert_eq!(diff.rows[0].left.number, Some(1));
        assert_eq!(diff.rows[0].right.number, Some(1));
        assert_eq!(diff.rows[1].left.number, Some(2));
        assert_eq!(diff.rows[1].left.kind, SideKind::Equal);
        assert_eq!(diff.rows[1].right.kind, SideKind::Empty);
        assert_eq!(diff.rows[2].left.number, Some(3));
        assert_eq!(diff.rows[2].right.number, Some(2));
        assert_eq!(diff.rows[2].left.text, "bravo");
    }

    #[test]
    fn ignored_lines_do_not_shift_original_line_numbers() {
        let options = CompareOptions {
            ignore_case: true,
            ..Default::default()
        };
        let diff =
            DiffResult::compute_with_options("Alpha\nbravo\n", "ALPHA\nnew\nBRAVO\n", options);

        assert_eq!(diff.stats.unchanged, 2);
        assert_eq!(diff.stats.added, 1);
        assert_eq!(diff.stats.changed, 0);
        assert_eq!(diff.rows.len(), 3);
        assert_eq!(diff.rows[0].left.number, Some(1));
        assert_eq!(diff.rows[0].right.number, Some(1));
        assert_eq!(diff.rows[0].left.text, "Alpha");
        assert_eq!(diff.rows[0].right.text, "ALPHA");
        assert_eq!(diff.rows[1].left.kind, SideKind::Empty);
        assert_eq!(diff.rows[1].right.number, Some(2));
        assert_eq!(diff.rows[1].right.text, "new");
        assert_eq!(diff.rows[2].left.number, Some(2));
        assert_eq!(diff.rows[2].right.number, Some(3));
        assert_eq!(diff.rows[2].left.text, "bravo");
        assert_eq!(diff.rows[2].right.text, "BRAVO");
        assert_eq!(emphasized(&diff.rows[2].left), "");
        assert_eq!(emphasized(&diff.rows[2].right), "");
    }

    #[test]
    fn extra_blank_lines_are_ignored_on_both_sides() {
        let options = CompareOptions {
            ignore_blank_lines: true,
            ..Default::default()
        };
        let diff =
            DiffResult::compute_with_options("alpha\n\nbravo\n", "alpha\n\n\nBRAVO\n", options);

        assert_eq!(diff.stats.added, 0);
        assert_eq!(diff.stats.removed, 0);
        assert_eq!(diff.stats.changed, 1);
        assert_eq!(diff.rows.len(), 4);
        assert_eq!(diff.rows[1].left.number, Some(2));
        assert_eq!(diff.rows[1].right.number, Some(2));
        assert_eq!(diff.rows[2].left.kind, SideKind::Empty);
        assert_eq!(diff.rows[2].right.number, Some(3));
        assert_eq!(diff.rows[3].left.number, Some(3));
        assert_eq!(diff.rows[3].right.number, Some(4));
    }

    #[test]
    fn change_hunks_group_contiguous_changed_rows() {
        let diff = DiffResult::compute(
            "alpha\nbravo\ncharlie\ndelta\necho\nfoxtrot\n",
            "alpha\nBRAVO\ncharlie\ndelta\nECHO\nfoxtrot\n",
        );
        assert_eq!(
            diff.change_hunks(),
            vec![
                ChangeHunk { start: 1, end: 2 },
                ChangeHunk { start: 4, end: 5 },
            ]
        );
    }

    #[test]
    fn adjacent_insertions_and_deletions_share_one_hunk() {
        let diff = DiffResult::compute(
            "alpha\nbravo\ncharlie\n",
            "alpha\nnew one\nnew two\ncharlie\n",
        );
        let hunks = diff.change_hunks();
        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].start, 1);
        assert_eq!(hunks[0].end, 3);
        assert_eq!(hunks[0].end - hunks[0].start, 2);
    }

    #[test]
    fn identical_diff_has_no_change_hunks() {
        let diff = DiffResult::compute("alpha\nbravo\n", "alpha\nbravo\n");
        assert!(diff.change_hunks().is_empty());
    }
}
