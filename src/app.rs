use std::collections::HashSet;

use gpui_kit::assets::IconName;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{InputEvent, Textarea, TextareaState};
use gpui_kit::component::link::Link;
use gpui_kit::component::{ActiveTheme as _, Disableable as _, Selectable as _};
use gpui_kit::prelude::*;
use gpui_kit::{EntityInputHandler as _, *};

use crate::diff::{CompareOptions, DiffResult, DiffRow, DiffSide, SideKind};
use crate::theme;

mod actions {
    gpui_kit::actions!(diff_nav, [NextChange, PreviousChange]);
}

/// Root key context for the application. Bindings scoped to it only fire while
/// this window has focus (and never while an `Input` context is focused).
const KEY_CONTEXT: &str = "TextDiff";
/// Added to the root context while either textarea has a live IME preedit, so
/// shortcuts stay out of the way of composition even if focus moves mid-edit.
const KEY_CONTEXT_COMPOSING: &str = "TextDiff ImeComposing";
/// `Input` is the context `gpui-base` installs on focused text controls
/// (including `Textarea`).
const KEY_BINDING_CONTEXT: &str = "TextDiff && !Input && !ImeComposing";

const DIFF_CONTEXT_ROWS: usize = 2;
const MIN_COLLAPSED_ROWS: usize = 2;
const DIFF_KEY_CONTEXT: &str = "DiffNav";

actions!(textdiff, [Compare, Swap, ClearAll]);

/// Register the app's key bindings. Call once after `gpui_kit::init`.
pub(crate) fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("n", actions::NextChange, Some(DIFF_KEY_CONTEXT)),
        KeyBinding::new("p", actions::PreviousChange, Some(DIFF_KEY_CONTEXT)),
    ]);

    let mut bindings = vec![
        // text-compare parity shortcut
        KeyBinding::new("ctrl-alt-c", Compare, Some(KEY_BINDING_CONTEXT)),
        KeyBinding::new("ctrl-alt-s", Swap, Some(KEY_BINDING_CONTEXT)),
        KeyBinding::new("ctrl-alt-r", ClearAll, Some(KEY_BINDING_CONTEXT)),
        KeyBinding::new("ctrl-enter", Compare, Some(KEY_BINDING_CONTEXT)),
    ];
    // Keep Ctrl+Enter working everywhere, and make Cmd+Enter the mac binding
    // advertised by tooltips (last registered wins for display).
    #[cfg(target_os = "macos")]
    bindings.push(KeyBinding::new(
        "cmd-enter",
        Compare,
        Some(KEY_BINDING_CONTEXT),
    ));
    cx.bind_keys(bindings);
}

enum Mode {
    Edit,
    Diff,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum MobilePane {
    Original,
    Modified,
}

/// One direct child of the diff scroll container.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DiffItem {
    Row(usize),
    Gap { start: usize, end: usize },
}

pub struct DiffApp {
    mode: Mode,
    focus_handle: FocusHandle,
    original: Entity<TextareaState>,
    modified: Entity<TextareaState>,
    diff: DiffResult,
    options: CompareOptions,
    mobile_pane: MobilePane,
    only_changes: bool,
    expanded_gaps: HashSet<usize>,
    active_hunk: Option<usize>,
    scroll_handle: ScrollHandle,
    diff_focus: FocusHandle,
    _subscriptions: Vec<Subscription>,
}

impl DiffApp {
    pub fn new(window: &mut Window, cx: &mut Context<'_, Self>) -> Self {
        let original = cx.new(|cx| {
            TextareaState::new(window, cx).placeholder("Paste the original text here…")
        });
        let modified = cx.new(|cx| {
            TextareaState::new(window, cx).placeholder("Paste the modified text here…")
        });
        let focus_handle = cx.focus_handle();
        focus_handle.focus(window, cx);

        let subscriptions = vec![
            cx.subscribe(&original, |_, _, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    cx.notify();
                }
            }),
            cx.subscribe(&modified, |_, _, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    cx.notify();
                }
            }),
        ];
        let diff_focus = cx.focus_handle();

        Self {
            mode: Mode::Edit,
            focus_handle,
            original,
            modified,
            diff: DiffResult::default(),
            options: CompareOptions::default(),
            mobile_pane: MobilePane::Original,
            only_changes: false,
            expanded_gaps: HashSet::new(),
            active_hunk: None,
            scroll_handle: ScrollHandle::new(),
            diff_focus,
            _subscriptions: subscriptions,
        }
    }

    fn can_compare(&self, cx: &App) -> bool {
        !self.original.read(cx).value().is_empty()
            || !self.modified.read(cx).value().is_empty()
    }

    /// True while either textarea has an active IME preedit.
    fn is_composing(&self, window: &mut Window, cx: &mut Context<'_, Self>) -> bool {
        self.original
            .update(cx, |state, cx| state.marked_text_range(window, cx).is_some())
            || self
                .modified
                .update(cx, |state, cx| state.marked_text_range(window, cx).is_some())
    }

    fn compare(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) {
        let original = self.original.read(cx).value().to_string();
        let modified = self.modified.read(cx).value().to_string();
        self.diff = DiffResult::compute_with_options(&original, &modified, self.options);
        self.mode = Mode::Diff;
        self.reset_navigation();
        self.diff_focus.focus(window, cx);
        cx.notify();
    }

    fn reset_navigation(&mut self) {
        self.active_hunk = None;
        self.expanded_gaps.clear();
        self.scroll_handle.set_offset(point(px(0.), px(0.)));
    }

    fn on_compare(&mut self, _: &Compare, window: &mut Window, cx: &mut Context<'_, Self>) {
        if self.can_compare(cx) {
            self.compare(window, cx);
        }
    }

    fn on_swap(&mut self, _: &Swap, window: &mut Window, cx: &mut Context<'_, Self>) {
        self.swap(window, cx);
    }

    fn on_clear_all(&mut self, _: &ClearAll, window: &mut Window, cx: &mut Context<'_, Self>) {
        if self.can_compare(cx) {
            self.clear(window, cx);
        }
    }

    fn back(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) {
        self.mode = Mode::Edit;
        self.focus_handle.focus(window, cx);
        cx.notify();
    }

    fn swap(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) {
        let original = self.original.read(cx).value().to_string();
        let modified = self.modified.read(cx).value().to_string();
        let original_state = self.original.clone();
        let modified_state = self.modified.clone();

        original_state.update(cx, |state, cx| state.set_value(modified, window, cx));
        modified_state.update(cx, |state, cx| state.set_value(original, window, cx));
        self.diff = DiffResult::default();
        self.reset_navigation();
        self.mode = Mode::Edit;
        self.focus_handle.focus(window, cx);
        cx.notify();
    }

    fn clear(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) {
        let original_state = self.original.clone();
        let modified_state = self.modified.clone();
        original_state.update(cx, |state, cx| state.set_value("", window, cx));
        modified_state.update(cx, |state, cx| state.set_value("", window, cx));
        self.diff = DiffResult::default();
        self.reset_navigation();
        self.mode = Mode::Edit;
        self.focus_handle.focus(window, cx);
        cx.notify();
    }

    fn load_example(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) {
        let original = "A diff tool should make change obvious.\nThis line was kept.\nThis line will be replaced.\nOne more line for context.";
        let modified = "A diff tool should make every change obvious.\nThis line was kept.\nThis line has been rewritten.\nOne more line for context.";
        let original_state = self.original.clone();
        let modified_state = self.modified.clone();

        original_state.update(cx, |state, cx| state.set_value(original, window, cx));
        modified_state.update(cx, |state, cx| state.set_value(modified, window, cx));
        self.mode = Mode::Edit;
        self.focus_handle.focus(window, cx);
        cx.notify();
    }

    fn line_count(&self, state: &Entity<TextareaState>, cx: &App) -> usize {
        state.read(cx).value().lines().count()
    }

    fn toggle_only_changes(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) {
        self.only_changes = !self.only_changes;
        self.diff_focus.focus(window, cx);
        cx.notify();
    }

    fn next_change(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) {
        self.jump_to_change(1, window, cx);
    }

    fn previous_change(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) {
        self.jump_to_change(-1, window, cx);
    }

    fn jump_to_change(&mut self, step: isize, window: &mut Window, cx: &mut Context<'_, Self>) {
        let hunks = self.diff.change_hunks();
        if hunks.is_empty() {
            self.active_hunk = None;
            cx.notify();
            return;
        }

        let next = match self.active_hunk.filter(|index| *index < hunks.len()) {
            Some(current) if step >= 0 => (current + 1) % hunks.len(),
            Some(current) => (current + hunks.len() - 1) % hunks.len(),
            None if step >= 0 => 0,
            None => hunks.len() - 1,
        };
        self.active_hunk = Some(next);

        let narrow = window.viewport_size().width < px(760.);
        let items = diff_display_items(&self.diff, self.only_changes, &self.expanded_gaps);
        if let Some(index) =
            display_index_for_row(&items, &self.diff.rows, hunks[next].start, narrow)
        {
            self.scroll_handle.scroll_to_item(index);
        }

        self.diff_focus.focus(window, cx);
        cx.notify();
    }

    fn expand_gap(&mut self, start: usize, window: &mut Window, cx: &mut Context<'_, Self>) {
        self.expanded_gaps.insert(start);
        self.diff_focus.focus(window, cx);
        cx.notify();
    }

    fn edit_view(
        &self,
        _window: &mut Window,
        cx: &mut Context<'_, Self>,
        narrow: bool,
    ) -> AnyElement {
        let original_lines = self.line_count(&self.original, cx);
        let modified_lines = self.line_count(&self.modified, cx);
        let can_compare = self.can_compare(cx);

        let panes = if narrow {
            let selected = match self.mobile_pane {
                MobilePane::Original => {
                    editor_card("Original", self.original.clone(), original_lines, true, cx)
                }
                MobilePane::Modified => {
                    editor_card("Modified", self.modified.clone(), modified_lines, true, cx)
                }
            };

            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_h_0()
                .gap_3()
                .child(self.mobile_tabs(original_lines, modified_lines, cx))
                .child(selected)
        } else {
            div()
                .flex()
                .flex_1()
                .flex_row()
                .min_h_0()
                .gap_4()
                .child(editor_card(
                    "Original",
                    self.original.clone(),
                    original_lines,
                    false,
                    cx,
                ))
                .child(editor_card(
                    "Modified",
                    self.modified.clone(),
                    modified_lines,
                    false,
                    cx,
                ))
        };

        let compare = Button::new("compare")
            .primary()
            .label("Compare")
            .disabled(!can_compare)
            .tooltip_with_action("Compare", &Compare, Some(KEY_CONTEXT))
            .on_click(cx.listener(|this, _, window, cx| this.compare(window, cx)));
        let example = Button::new("example")
            .ghost()
            .label("Load example")
            .on_click(cx.listener(|this, _, window, cx| this.load_example(window, cx)));
        let swap = Button::new("swap")
            .ghost()
            .icon(IconName::ArrowLeftRight)
            .label("Swap")
            .tooltip_with_action("Swap original and modified", &Swap, Some(KEY_CONTEXT))
            .on_click(cx.listener(|this, _, window, cx| this.swap(window, cx)));
        let clear = Button::new("clear")
            .danger()
            .icon(IconName::Trash)
            .label("Clear all")
            .disabled(!can_compare)
            .tooltip_with_action("Clear both panes", &ClearAll, Some(KEY_CONTEXT))
            .on_click(cx.listener(|this, _, window, cx| this.clear(window, cx)));

        let actions = if narrow {
            div()
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(compare)
                        .child(example),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_end()
                        .gap_2()
                        .child(swap)
                        .child(clear),
                )
        } else {
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(compare)
                        .child(example),
                )
                .child(div().flex().items_center().gap_2().child(swap).child(clear))
        };

        let content = div()
            .flex()
            .flex_col()
            .flex_1()
            .w_full()
            .min_w_0()
            .min_h_0()
            .px(if narrow { px(16.) } else { px(28.) })
            .pt(if narrow { px(18.) } else { px(26.) })
            .pb(if narrow { px(12.) } else { px(18.) })
            .gap_4()
            .child(panes)
            .child(self.options_row(cx))
            .child(actions);

        content.into_any_element()
    }

    fn options_row(&self, cx: &mut Context<'_, Self>) -> Div {
        let muted = cx.theme().muted_foreground;
        div()
            .flex()
            .items_center()
            .flex_wrap()
            .gap_2()
            .child(
                div()
                    .text_size(px(11.))
                    .text_color(muted)
                    .child("Ignore"),
            )
            .child(self.option_toggle("opt-case", "Case", self.options.ignore_case, cx))
            .child(self.option_toggle(
                "opt-whitespace",
                "Whitespace",
                self.options.ignore_whitespace,
                cx,
            ))
            .child(self.option_toggle(
                "opt-blank-lines",
                "Blank lines",
                self.options.ignore_blank_lines,
                cx,
            ))
    }

    fn option_toggle(
        &self,
        id: &'static str,
        label: &'static str,
        active: bool,
        cx: &mut Context<'_, Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .id(id)
            .flex()
            .items_center()
            .h(px(26.))
            .px(px(10.))
            .rounded(px(6.))
            .border_1()
            .border_color(theme.border)
            .cursor_pointer()
            .when(active, |this| {
                this.bg(theme.group_box).text_color(theme.foreground)
            })
            .when(!active, |this| this.text_color(theme.muted_foreground))
            .child(div().text_size(px(11.)).child(label))
            .on_click(cx.listener(move |this, _, _, cx| {
                match id {
                    "opt-case" => this.options.ignore_case = !this.options.ignore_case,
                    "opt-whitespace" => {
                        this.options.ignore_whitespace = !this.options.ignore_whitespace
                    }
                    "opt-blank-lines" => {
                        this.options.ignore_blank_lines = !this.options.ignore_blank_lines
                    }
                    _ => {}
                }
                cx.notify();
            }))
    }

    fn mobile_tabs(
        &self,
        original_lines: usize,
        modified_lines: usize,
        cx: &mut Context<'_, Self>,
    ) -> Div {
        let group_box = cx.theme().group_box;
        let muted = cx.theme().muted;
        let muted_foreground = cx.theme().muted_foreground;
        let tab = |id: &'static str,
                   label: &'static str,
                   count: usize,
                   selected: bool,
                   cx: &mut Context<'_, Self>| {
            div()
                .id(id)
                .flex()
                .flex_1()
                .items_center()
                .justify_center()
                .gap_2()
                .h(px(32.))
                .rounded(px(6.))
                .cursor_pointer()
                .when(selected, |this| this.bg(group_box))
                .when(!selected, |this| this.text_color(muted_foreground))
                .child(
                    div()
                        .text_sm()
                        .when(selected, |this| this.font_weight(FontWeight::MEDIUM))
                        .child(label),
                )
                .child(
                    div()
                        .text_size(px(11.))
                        .text_color(muted_foreground)
                        .child(count.to_string()),
                )
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.mobile_pane = match id {
                        "tab-original" => MobilePane::Original,
                        _ => MobilePane::Modified,
                    };
                    cx.notify();
                }))
        };

        div()
            .flex()
            .items_center()
            .gap_1()
            .p(px(2.))
            .rounded(px(8.))
            .bg(muted)
            .child(tab(
                "tab-original",
                "Original",
                original_lines,
                self.mobile_pane == MobilePane::Original,
                cx,
            ))
            .child(tab(
                "tab-modified",
                "Modified",
                modified_lines,
                self.mobile_pane == MobilePane::Modified,
                cx,
            ))
    }

    fn diff_view(
        &self,
        _window: &mut Window,
        cx: &mut Context<'_, Self>,
        narrow: bool,
    ) -> AnyElement {
        let border = cx.theme().border;
        let group_box = cx.theme().group_box;
        let muted_foreground = cx.theme().muted_foreground;
        let stats = self.diff.stats;
        let rows = &self.diff.rows;
        let filters = self.options.active_labels();
        let filter_label = if filters.is_empty() {
            String::new()
        } else {
            format!(" · ignoring {}", filters.join(", "))
        };
        let hunks = self.diff.change_hunks();
        let items = diff_display_items(&self.diff, self.only_changes, &self.expanded_gaps);
        let show_empty_state = rows.is_empty() || (self.only_changes && hunks.is_empty());

        let body: Vec<AnyElement> = if show_empty_state {
            vec![empty_state(cx).into_any_element()]
        } else if narrow {
            let mut elements = Vec::new();
            for item in &items {
                match *item {
                    DiffItem::Row(index) => {
                        let row = &rows[index];
                        let (show_left, show_right) = unified_line_sides(row);
                        if show_left {
                            elements.push(unified_row(&row.left, cx).into_any_element());
                        }
                        if show_right {
                            elements.push(unified_row(&row.right, cx).into_any_element());
                        }
                    }
                    DiffItem::Gap { start, end } => {
                        elements.push(gap_row(start, end, false, cx));
                    }
                }
            }
            elements
        } else {
            items
                .iter()
                .map(|item| match *item {
                    DiffItem::Row(index) => diff_row(&rows[index], cx).into_any_element(),
                    DiffItem::Gap { start, end } => gap_row(start, end, true, cx),
                })
                .collect()
        };

        let diff_header = if narrow {
            div()
                .flex()
                .items_center()
                .h(px(36.))
                .px(px(14.))
                .border_b_1()
                .border_color(border)
                .text_xs()
                .text_color(muted_foreground)
                .child("Unified diff")
                .into_any_element()
        } else {
            div()
                .flex()
                .items_center()
                .h(px(40.))
                .border_b_1()
                .border_color(border)
                .child(panel_header("Original", cx))
                .child(div().w(px(1.)).h_full().bg(border))
                .child(panel_header("Modified", cx))
                .into_any_element()
        };

        let can_navigate = !hunks.is_empty();
        let position_label = match self.active_hunk.filter(|index| *index < hunks.len()) {
            Some(index) => format!("{} of {}", index + 1, hunks.len()),
            None if hunks.is_empty() => "No changes".to_string(),
            None => format!("— of {}", hunks.len()),
        };

        let only_changes = Button::new("only-changes")
            .ghost()
            .label("Only changes")
            .selected(self.only_changes)
            .toggled(self.only_changes)
            .on_click(cx.listener(|this, _, window, cx| this.toggle_only_changes(window, cx)));
        let previous = Button::new("previous-change")
            .ghost()
            .label("← Previous")
            .disabled(!can_navigate)
            .on_click(cx.listener(|this, _, window, cx| this.previous_change(window, cx)));
        let next = Button::new("next-change")
            .ghost()
            .label("Next →")
            .disabled(!can_navigate)
            .on_click(cx.listener(|this, _, window, cx| this.next_change(window, cx)));

        div()
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .px(if narrow { px(16.) } else { px(28.) })
            .pt(px(20.))
            .pb(if narrow { px(12.) } else { px(18.) })
            .gap_3()
            .track_focus(&self.diff_focus)
            .key_context(DIFF_KEY_CONTEXT)
            .on_action(
                cx.listener(|this, _: &actions::NextChange, window, cx| {
                    this.next_change(window, cx)
                }),
            )
            .on_action(
                cx.listener(|this, _: &actions::PreviousChange, window, cx| {
                    this.previous_change(window, cx)
                }),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .when(narrow, |this| this.flex_col().items_start().gap_2())
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(
                                Button::new("back").ghost().label("← Edit").on_click(
                                    cx.listener(|this, _, window, cx| this.back(window, cx)),
                                ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .child(
                                        div()
                                            .text_lg()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .child("Comparison"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(muted_foreground)
                                            .child(format!(
                                                "{} rows{}",
                                                rows.len(),
                                                filter_label
                                            )),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .items_center()
                            .gap_3()
                            .when(stats.added > 0, |this| {
                                this.child(stat("added", stats.added, theme::GREEN))
                            })
                            .when(stats.removed > 0, |this| {
                                this.child(stat("removed", stats.removed, theme::RED))
                            })
                            .when(stats.changed > 0, |this| {
                                this.child(stat("changed", stats.changed, theme::ACCENT))
                            })
                            .when(stats.unchanged > 0, |this| {
                                this.child(stat("unchanged", stats.unchanged, theme::MUTED))
                            }),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .flex_wrap()
                    .gap_2()
                    .when(narrow, |this| this.flex_col().items_start())
                    .child(only_changes)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(previous)
                            .child(
                                div()
                                    .min_w(px(64.))
                                    .text_center()
                                    .text_xs()
                                    .text_color(muted_foreground)
                                    .child(position_label),
                            )
                            .child(next),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h_0()
                    .rounded(px(10.))
                    .border_1()
                    .border_color(border)
                    .bg(group_box)
                    .overflow_hidden()
                    .child(diff_header)
                    .child(
                        div()
                            .id("diff-scroll")
                            .flex()
                            .flex_col()
                            .flex_1()
                            .min_h_0()
                            .overflow_scroll()
                            .track_scroll(&self.scroll_handle)
                            .children(body),
                    ),
            )
            .into_any_element()
    }
}

impl Render for DiffApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let background = cx.theme().background;
        let foreground = cx.theme().foreground;
        let narrow = window.viewport_size().width < px(760.);
        let content = match self.mode {
            Mode::Edit => self.edit_view(window, cx, narrow),
            Mode::Diff => self.diff_view(window, cx, narrow),
        };
        let key_context = if self.is_composing(window, cx) {
            KEY_CONTEXT_COMPOSING
        } else {
            KEY_CONTEXT
        };

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(background)
            .text_color(foreground)
            .track_focus(&self.focus_handle)
            .key_context(key_context)
            .on_action(cx.listener(Self::on_compare))
            .on_action(cx.listener(Self::on_swap))
            .on_action(cx.listener(Self::on_clear_all))
            .child(header(cx, narrow))
            .child(content)
            .child(footer(cx))
    }
}

fn header(cx: &App, narrow: bool) -> Div {
    let theme = cx.theme();
    div()
        .flex()
        .items_center()
        .justify_between()
        .h(if narrow { px(52.) } else { px(60.) })
        .px(if narrow { px(16.) } else { px(28.) })
        .border_b_1()
        .border_color(theme.border)
        .child(
            div().flex().items_center().gap_3().child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Text Diff"),
            ),
        )
        .child(
            div().flex().items_center().gap_4().child(
                Link::new("github")
                    .href("https://github.com/tnmyk/textdiff")
                    .text_sm()
                    .child("GitHub"),
            ),
        )
}

fn footer(cx: &App) -> Div {
    let theme = cx.theme();
    div()
        .flex()
        .items_center()
        .justify_center()
        .h(px(32.))
        .border_t_1()
        .border_color(theme.border)
        .text_size(px(11.))
        .text_color(theme.muted_foreground)
        .child("Local only. Nothing is uploaded.")
}

fn editor_card(
    label: &'static str,
    state: Entity<TextareaState>,
    lines: usize,
    compact: bool,
    cx: &App,
) -> Div {
    let theme = cx.theme();
    let card = div()
        .flex()
        .flex_col()
        .w_full()
        .min_w_0()
        .min_h_0()
        .rounded(px(10.))
        .border_1()
        .border_color(theme.border)
        .bg(theme.group_box)
        .overflow_hidden()
        .when(!compact, |this| {
            this.child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .h(px(44.))
                    .px(px(16.))
                    .border_b_1()
                    .border_color(theme.border)
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child(label),
                    )
                    .child(
                        div()
                            .text_size(px(11.))
                            .text_color(theme.muted_foreground)
                            .child(format!("{lines} lines")),
                    ),
            )
        })
        .child(
            div().flex_1().min_h_0().p(px(12.)).child(
                Textarea::new(&state)
                    .w_full()
                    .h(relative(1.))
                    .appearance(false)
                    .bordered(false)
                    .aria_label(label),
            ),
        );

    card.flex_1().min_h_0()
}

fn panel_header(label: &'static str, cx: &App) -> Div {
    let theme = cx.theme();
    div()
        .flex()
        .flex_1()
        .items_center()
        .h_full()
        .px(px(14.))
        .text_size(px(11.))
        .text_color(theme.muted_foreground)
        .child(label)
}

fn empty_state(cx: &App) -> Div {
    let muted = cx.theme().muted_foreground;
    div()
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .w_full()
        .h(px(120.))
        .text_sm()
        .text_color(muted)
        .child("The two texts are identical.")
}

/// Build the rows shown by the diff body.
///
/// In "Only changes" mode each hunk keeps [`DIFF_CONTEXT_ROWS`] unchanged rows on
/// either side. Unchanged runs with at least [`MIN_COLLAPSED_ROWS`] hidden rows
/// become one gap item; smaller remainders are shown inline to avoid noisy
/// one-line "Show" buttons.
fn diff_display_items(
    diff: &DiffResult,
    only_changes: bool,
    expanded_gaps: &HashSet<usize>,
) -> Vec<DiffItem> {
    if !only_changes {
        return (0..diff.rows.len()).map(DiffItem::Row).collect();
    }

    let hunks = diff.change_hunks();
    if hunks.is_empty() {
        return Vec::new();
    }

    let mut intervals: Vec<(usize, usize)> = Vec::with_capacity(hunks.len());
    for hunk in hunks {
        let start = hunk.start.saturating_sub(DIFF_CONTEXT_ROWS);
        let end = (hunk.end + DIFF_CONTEXT_ROWS).min(diff.rows.len());
        if let Some(last) = intervals.last_mut()
            && start <= last.1
        {
            last.1 = last.1.max(end);
        } else {
            intervals.push((start, end));
        }
    }

    let mut items = Vec::new();
    let mut cursor = 0;
    for (start, end) in intervals {
        if cursor < start {
            push_unchanged_rows(&mut items, cursor, start, expanded_gaps);
        }
        items.extend((start..end).map(DiffItem::Row));
        cursor = end;
    }
    if cursor < diff.rows.len() {
        push_unchanged_rows(&mut items, cursor, diff.rows.len(), expanded_gaps);
    }

    items
}

fn push_unchanged_rows(
    items: &mut Vec<DiffItem>,
    start: usize,
    end: usize,
    expanded_gaps: &HashSet<usize>,
) {
    if start >= end {
        return;
    }
    if expanded_gaps.contains(&start) || end - start < MIN_COLLAPSED_ROWS {
        items.extend((start..end).map(DiffItem::Row));
    } else {
        items.push(DiffItem::Gap { start, end });
    }
}

/// Map a diff row index to the index of its direct child in the scroll
/// container. Unified mode can render up to two lines per diff row.
fn display_index_for_row(
    items: &[DiffItem],
    rows: &[DiffRow],
    row_index: usize,
    unified: bool,
) -> Option<usize> {
    let mut index = 0;
    for item in items {
        match *item {
            DiffItem::Row(row) => {
                if row == row_index {
                    return Some(index);
                }
                index += if unified {
                    unified_line_count(&rows[row])
                } else {
                    1
                };
            }
            DiffItem::Gap { .. } => index += 1,
        }
    }
    None
}

fn unified_line_count(row: &DiffRow) -> usize {
    let (show_left, show_right) = unified_line_sides(row);
    usize::from(show_left) + usize::from(show_right)
}

/// Equal rows render once. Change rows render each populated side, removed
/// before added.
fn unified_line_sides(row: &DiffRow) -> (bool, bool) {
    let show_left = row.left.kind != SideKind::Empty;
    let show_right = row.right.kind != SideKind::Empty && row.left.kind != SideKind::Equal;
    (show_left, show_right)
}

fn gap_row(start: usize, end: usize, split: bool, cx: &mut Context<'_, DiffApp>) -> AnyElement {
    let theme = cx.theme();
    let count = end.saturating_sub(start);
    let label = if count == 1 {
        "Show 1 unchanged line".to_string()
    } else {
        format!("Show {count} unchanged lines")
    };

    div()
        .id(("gap", start))
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .w_full()
        .when(split, |this| this.min_w(px(980.)))
        .h(px(26.))
        .bg(theme.muted)
        .text_size(px(11.))
        .text_color(theme.muted_foreground)
        .cursor_pointer()
        .hover(|this| this.bg(theme.border))
        .child(label)
        .on_click(cx.listener(move |this, _, window, cx| this.expand_gap(start, window, cx)))
        .into_any_element()
}

fn unified_row(side: &DiffSide, cx: &App) -> Div {
    let theme = cx.theme();
    let (bg, bar, sign, sign_color, text_color) = match side.kind {
        SideKind::Equal => (None, None, " ", theme.muted_foreground, theme.foreground),
        SideKind::Removed => (
            Some(theme::color(theme::RED_BG)),
            Some(theme::color(theme::RED)),
            "−",
            theme::color(theme::RED),
            theme.foreground,
        ),
        SideKind::Added => (
            Some(theme::color(theme::GREEN_BG)),
            Some(theme::color(theme::GREEN)),
            "+",
            theme::color(theme::GREEN),
            theme.foreground,
        ),
        SideKind::Empty => return div(),
    };

    let mut row = div().flex().flex_none().w_full().min_h(px(22.));
    if let Some(bg) = bg {
        row = row.bg(bg);
    }
    if let Some(bar) = bar {
        row = row.child(div().w(px(2.)).h_full().bg(bar));
    }

    row.child(
        div()
            .w(px(20.))
            .pt(px(2.))
            .text_center()
            .text_size(px(12.))
            .text_color(sign_color)
            .child(sign),
    )
    .child(
        div()
            .w(px(40.))
            .pr(px(8.))
            .pt(px(2.))
            .text_right()
            .text_size(px(11.))
            .text_color(theme.muted_foreground)
            .child(
                side.number
                    .map(|number| number.to_string())
                    .unwrap_or_default(),
            ),
    )
    .child(
        div()
            .flex_1()
            .min_w_0()
            .pt(px(2.))
            .pr(px(10.))
            .whitespace_normal()
            .font_family(theme.mono_font_family.clone())
            .text_size(px(13.))
            .line_height(px(18.))
            .text_color(text_color)
            .when(side.text.is_empty(), |this| this.child(" "))
            .when(!side.text.is_empty(), |this| this.child(styled_line(side))),
    )
}

fn diff_row(row: &DiffRow, cx: &App) -> Div {
    let theme = cx.theme();
    div()
        .flex()
        .flex_none()
        .w_full()
        .min_w(px(980.))
        .min_h(px(22.))
        .child(side_cell(&row.left, true, cx))
        .child(div().w(px(1.)).bg(theme.border))
        .child(side_cell(&row.right, false, cx))
}

fn side_cell(side: &DiffSide, is_left: bool, cx: &App) -> Div {
    let theme = cx.theme();
    let (bg, bar, text_color) = match side.kind {
        SideKind::Equal => (None, None, theme.foreground),
        SideKind::Removed => (
            Some(theme::color(theme::RED_BG)),
            Some(theme::color(theme::RED)),
            theme.foreground,
        ),
        SideKind::Added => (
            Some(theme::color(theme::GREEN_BG)),
            Some(theme::color(theme::GREEN)),
            theme.foreground,
        ),
        SideKind::Empty => (
            Some(theme::color(theme::PANEL_ALT)),
            None,
            theme.muted_foreground,
        ),
    };

    let mut cell = div()
        .flex()
        .flex_1()
        .min_w_0()
        .h_full()
        .items_start()
        .overflow_hidden();

    if let Some(bg) = bg {
        cell = cell.bg(bg);
    }

    if is_left {
        if let Some(bar) = bar {
            cell = cell.child(div().w(px(2.)).h_full().bg(bar));
        }
    }

    cell = cell
        .child(
            div()
                .w(px(42.))
                .pr(px(10.))
                .pt(px(2.))
                .text_right()
                .text_size(px(11.))
                .text_color(theme.muted_foreground)
                .child(
                    side.number
                        .map(|number| number.to_string())
                        .unwrap_or_default(),
                ),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .pt(px(2.))
                .pr(px(10.))
                .whitespace_nowrap()
                .font_family(theme.mono_font_family.clone())
                .text_size(px(13.))
                .line_height(px(18.))
                .text_color(text_color)
                .when(side.text.is_empty(), |this| this.child(" "))
                .when(!side.text.is_empty(), |this| this.child(styled_line(side))),
        );

    if !is_left {
        if let Some(bar) = bar {
            cell = cell.child(div().w(px(2.)).h_full().bg(bar));
        }
    }

    cell
}

fn styled_line(side: &DiffSide) -> StyledText {
    let strong = match side.kind {
        SideKind::Removed => theme::color(theme::RED_STRONG),
        SideKind::Added => theme::color(theme::GREEN_STRONG),
        _ => theme::color(theme::ACCENT),
    };
    let mut highlights = Vec::new();
    let mut offset = 0;
    for segment in &side.segments {
        if segment.emphasis {
            highlights.push((
                offset..offset + segment.text.len(),
                HighlightStyle {
                    background_color: Some(strong),
                    color: Some(theme::color(0xFFFFFF)),
                    ..Default::default()
                },
            ));
        }
        offset += segment.text.len();
    }

    StyledText::new(side.text.clone()).with_highlights(highlights)
}

fn stat(label: &'static str, count: usize, tint: u32) -> Div {
    let color = theme::color(tint);
    div()
        .flex()
        .items_center()
        .gap_1()
        .text_size(px(11.))
        .child(div().text_color(color).child(count.to_string()))
        .child(div().text_color(theme::color(theme::MUTED)).child(label))
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{DiffItem, diff_display_items, display_index_for_row, unified_line_sides};
    use crate::diff::DiffResult;

    fn ten_line_diff() -> DiffResult {
        DiffResult::compute(
            "alpha\nbravo\ncharlie\ndelta\necho\nfoxtrot\ngolf\nhotel\nindia\njuliet\n",
            "alpha\nbravo\ncharlie\ndelta\nECHO\nfoxtrot\ngolf\nhotel\nindia\njuliet\n",
        )
    }

    #[test]
    fn only_changes_keeps_two_context_rows_and_collapses_the_rest() {
        let diff = ten_line_diff();
        let items = diff_display_items(&diff, true, &HashSet::new());

        assert_eq!(
            items,
            vec![
                DiffItem::Gap { start: 0, end: 2 },
                DiffItem::Row(2),
                DiffItem::Row(3),
                DiffItem::Row(4),
                DiffItem::Row(5),
                DiffItem::Row(6),
                DiffItem::Gap { start: 7, end: 10 },
            ]
        );
    }

    #[test]
    fn expanding_a_gap_reveals_its_rows() {
        let diff = ten_line_diff();
        let expanded = HashSet::from([0]);
        let items = diff_display_items(&diff, true, &expanded);

        assert_eq!(items[0], DiffItem::Row(0));
        assert_eq!(items[1], DiffItem::Row(1));
        assert_eq!(items[2], DiffItem::Row(2));
        assert!(!items.contains(&DiffItem::Gap { start: 0, end: 2 }));
        assert!(items.contains(&DiffItem::Gap { start: 7, end: 10 }));
    }

    #[test]
    fn single_hidden_rows_stay_visible() {
        let diff = DiffResult::compute(
            "alpha\nbravo\ncharlie\ndelta\n",
            "alpha\nbravo\ncharlie\nDELTA\n",
        );
        let items = diff_display_items(&diff, true, &HashSet::new());

        assert_eq!(items[0], DiffItem::Row(0));
        assert!(
            !items
                .iter()
                .any(|item| matches!(item, DiffItem::Gap { .. }))
        );
    }

    #[test]
    fn unified_scroll_index_accounts_for_two_line_rows() {
        let diff = DiffResult::compute(
            "alpha\nbravo\ncharlie\ndelta\n",
            "alpha\nBRAVO\ncharlie\ndelta\n",
        );
        let items = diff_display_items(&diff, false, &HashSet::new());

        // Row 0 is one equal line, row 1 renders removed + added, row 2 one
        // equal line, so row 3 starts at unified line index 4.
        assert_eq!(display_index_for_row(&items, &diff.rows, 3, true), Some(4));
        assert_eq!(display_index_for_row(&items, &diff.rows, 3, false), Some(3));
    }

    #[test]
    fn equal_rows_render_once_in_unified_mode() {
        let diff = ten_line_diff();
        let (show_left, show_right) = unified_line_sides(&diff.rows[0]);
        assert!(show_left);
        assert!(!show_right);

        let (show_left, show_right) = unified_line_sides(&diff.rows[4]);
        assert!(show_left);
        assert!(show_right);
    }
}

#[cfg(test)]
mod shortcut_tests {
    use gpui::{
        Entity, EntityInputHandler as _, Focusable as _, KeyBindingContextPredicate, KeyContext,
        TestAppContext, VisualTestContext,
    };

    use super::{DiffApp, Mode, KEY_BINDING_CONTEXT, KEY_CONTEXT, KEY_CONTEXT_COMPOSING, init};

    #[cfg(target_os = "macos")]
    const COMPARE_KEY: &str = "cmd-enter";
    #[cfg(not(target_os = "macos"))]
    const COMPARE_KEY: &str = "ctrl-enter";

    fn setup(cx: &mut TestAppContext) -> (Entity<DiffApp>, &mut VisualTestContext) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            init(cx);
        });
        cx.add_window_view(|window, cx| DiffApp::new(window, cx))
    }

    fn render(cx: &mut VisualTestContext) {
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
    }

    fn mode(view: &Entity<DiffApp>, cx: &VisualTestContext) -> Mode {
        cx.read(|cx| view.read(cx).mode)
    }

    fn original_value(view: &Entity<DiffApp>, cx: &VisualTestContext) -> String {
        cx.read(|cx| view.read(cx).original.read(cx).value().to_string())
    }

    fn modified_value(view: &Entity<DiffApp>, cx: &VisualTestContext) -> String {
        cx.read(|cx| view.read(cx).modified.read(cx).value().to_string())
    }

    fn set_values(
        view: &Entity<DiffApp>,
        cx: &mut VisualTestContext,
        original: &str,
        modified: &str,
    ) {
        cx.update(|window, cx| {
            view.update(cx, |app, cx| {
                app.original
                    .update(cx, |state, cx| state.set_value(original, window, cx));
                app.modified
                    .update(cx, |state, cx| state.set_value(modified, window, cx));
            });
        });
    }

    #[gpui::test]
    fn shortcuts_compare_swap_and_clear(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx);
        render(cx);
        set_values(&view, cx, "one\ntwo", "one\nthree");

        cx.simulate_keystrokes(COMPARE_KEY);
        assert_eq!(mode(&view, cx), Mode::Diff);

        cx.simulate_keystrokes("ctrl-alt-s");
        assert_eq!(mode(&view, cx), Mode::Edit);
        assert_eq!(original_value(&view, cx), "one\nthree");
        assert_eq!(modified_value(&view, cx), "one\ntwo");

        cx.simulate_keystrokes("ctrl-alt-r");
        assert!(original_value(&view, cx).is_empty());
        assert!(modified_value(&view, cx).is_empty());

        set_values(&view, cx, "a", "b");
        cx.simulate_keystrokes("ctrl-alt-c");
        assert_eq!(mode(&view, cx), Mode::Diff);
    }

    #[gpui::test]
    fn compare_from_textarea_restores_root_shortcuts(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx);
        render(cx);
        set_values(&view, cx, "one\ntwo", "one\nthree");

        cx.update(|window, cx| {
            view.update(cx, |app, cx| {
                app.original.read(cx).focus_handle(cx).focus(window, cx);
                app.compare(window, cx);
            });
        });
        assert_eq!(mode(&view, cx), Mode::Diff);

        cx.simulate_keystrokes("ctrl-alt-s");
        assert_eq!(mode(&view, cx), Mode::Edit);
        assert_eq!(original_value(&view, cx), "one\nthree");
    }

    #[gpui::test]
    fn shortcuts_stay_off_focused_textarea_and_ime_preedit(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx);
        render(cx);
        set_values(&view, cx, "one\ntwo", "one\nthree");

        cx.update(|window, cx| {
            view.update(cx, |app, cx| {
                app.original.read(cx).focus_handle(cx).focus(window, cx);
            });
        });
        cx.simulate_keystrokes(COMPARE_KEY);
        cx.simulate_keystrokes("ctrl-alt-c");
        cx.simulate_keystrokes("ctrl-alt-s");
        cx.simulate_keystrokes("ctrl-alt-r");
        assert_eq!(mode(&view, cx), Mode::Edit);
        assert_eq!(original_value(&view, cx), "one\ntwo");
        assert_eq!(modified_value(&view, cx), "one\nthree");

        // An active IME preedit keeps shortcuts off even after focus leaves the
        // field mid-composition.
        cx.update(|window, cx| {
            view.update(cx, |app, cx| {
                app.original.update(cx, |state, cx| {
                    state.replace_and_mark_text_in_range(None, "に", None, window, cx);
                });
            });
        });
        cx.update(|window, cx| window.blur(cx));
        cx.simulate_keystrokes(COMPARE_KEY);
        assert_eq!(mode(&view, cx), Mode::Edit);
        assert_eq!(modified_value(&view, cx), "one\nthree");

        cx.update(|window, cx| {
            view.update(cx, |app, cx| {
                app.original
                    .update(cx, |state, cx| state.unmark_text(window, cx));
                app.focus_handle.focus(window, cx);
            });
        });
        cx.simulate_keystrokes(COMPARE_KEY);
        assert_eq!(mode(&view, cx), Mode::Diff);
    }

    #[test]
    fn binding_predicate_excludes_text_input_and_ime_contexts() {
        let predicate = KeyBindingContextPredicate::parse(KEY_BINDING_CONTEXT).unwrap();
        let root = KeyContext::parse(KEY_CONTEXT).unwrap();

        assert_eq!(predicate.depth_of(&[root.clone()]), Some(1));
        assert_eq!(
            predicate.depth_of(&[root.clone(), KeyContext::parse("Input").unwrap()]),
            None
        );
        assert_eq!(
            predicate.depth_of(&[
                root,
                KeyContext::parse(KEY_CONTEXT_COMPOSING).unwrap(),
            ]),
            None
        );
    }
}
