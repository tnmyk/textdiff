use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{InputEvent, Textarea, TextareaState};
use gpui_kit::component::link::Link;
use gpui_kit::assets::IconName;
use gpui_kit::component::{ActiveTheme as _, Disableable as _};
use gpui_kit::prelude::*;
use gpui_kit::*;

use crate::diff::{DiffResult, DiffRow, DiffSide, SideKind};
use crate::file_io::{self, LoadedText};
use crate::theme;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Edit,
    Diff,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum MobilePane {
    Original,
    Modified,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum FileTarget {
    Original,
    Modified,
}

impl FileTarget {
    #[cfg_attr(not(target_family = "wasm"), allow(dead_code))]
    pub(crate) fn from_id(id: &str) -> Option<Self> {
        match id {
            "original" => Some(Self::Original),
            "modified" => Some(Self::Modified),
            _ => None,
        }
    }

    #[cfg_attr(not(target_family = "wasm"), allow(dead_code))]
    fn id(self) -> &'static str {
        match self {
            Self::Original => "original",
            Self::Modified => "modified",
        }
    }

    fn button_id(self) -> &'static str {
        match self {
            Self::Original => "open-original",
            Self::Modified => "open-modified",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Original => "Original",
            Self::Modified => "Modified",
        }
    }
}

#[derive(Clone)]
struct FileInfo {
    name: String,
    bytes: usize,
}

pub struct DiffApp {
    mode: Mode,
    original: Entity<TextareaState>,
    modified: Entity<TextareaState>,
    original_file: Option<FileInfo>,
    modified_file: Option<FileInfo>,
    file_error: Option<SharedString>,
    diff: DiffResult,
    mobile_pane: MobilePane,
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

        let subscriptions = vec![
            cx.subscribe(&original, |this, _, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    this.original_file = None;
                    cx.notify();
                }
            }),
            cx.subscribe(&modified, |this, _, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    this.modified_file = None;
                    cx.notify();
                }
            }),
        ];

        Self {
            mode: Mode::Edit,
            original,
            modified,
            original_file: None,
            modified_file: None,
            file_error: None,
            diff: DiffResult::default(),
            mobile_pane: MobilePane::Original,
            _subscriptions: subscriptions,
        }
    }

    fn compare(&mut self, _window: &mut Window, cx: &mut Context<'_, Self>) {
        let original = self.original.read(cx).value().to_string();
        let modified = self.modified.read(cx).value().to_string();
        self.diff = DiffResult::compute(&original, &modified);
        self.mode = Mode::Diff;
        cx.notify();
    }

    fn back(&mut self, _window: &mut Window, cx: &mut Context<'_, Self>) {
        self.mode = Mode::Edit;
        cx.notify();
    }

    fn swap(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) {
        let original = self.original.read(cx).value().to_string();
        let modified = self.modified.read(cx).value().to_string();
        let original_state = self.original.clone();
        let modified_state = self.modified.clone();

        original_state.update(cx, |state, cx| state.set_value(modified, window, cx));
        modified_state.update(cx, |state, cx| state.set_value(original, window, cx));
        std::mem::swap(&mut self.original_file, &mut self.modified_file);
        self.diff = DiffResult::default();
        self.mode = Mode::Edit;
        cx.notify();
    }

    fn clear(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) {
        let original_state = self.original.clone();
        let modified_state = self.modified.clone();
        original_state.update(cx, |state, cx| state.set_value("", window, cx));
        modified_state.update(cx, |state, cx| state.set_value("", window, cx));
        self.original_file = None;
        self.modified_file = None;
        self.file_error = None;
        self.diff = DiffResult::default();
        self.mode = Mode::Edit;
        cx.notify();
    }

    fn load_example(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) {
        let original = "A diff tool should make change obvious.\nThis line was kept.\nThis line will be replaced.\nOne more line for context.";
        let modified = "A diff tool should make every change obvious.\nThis line was kept.\nThis line has been rewritten.\nOne more line for context.";
        let original_state = self.original.clone();
        let modified_state = self.modified.clone();

        original_state.update(cx, |state, cx| state.set_value(original, window, cx));
        modified_state.update(cx, |state, cx| state.set_value(modified, window, cx));
        self.original_file = None;
        self.modified_file = None;
        self.file_error = None;
        self.mode = Mode::Edit;
        cx.notify();
    }

    fn open_file(&mut self, target: FileTarget, cx: &mut Context<'_, Self>) {
        self.file_error = None;
        cx.notify();

        #[cfg(not(target_family = "wasm"))]
        {
            let receiver = cx.prompt_for_paths(PathPromptOptions {
                files: true,
                directories: false,
                multiple: false,
                prompt: Some("Open a text file".into()),
            });

            cx.spawn(async move |this, cx| {
                let selection = match receiver.await {
                    Ok(selection) => selection,
                    Err(_) => {
                        let _ = this.update(cx, |app, cx| {
                            app.set_file_error("The file picker closed unexpectedly.", cx)
                        });
                        return;
                    }
                };

                match selection {
                    Ok(Some(paths)) => {
                        if let Some(path) = paths.into_iter().next() {
                            let loaded = file_io::read_text_file(&path);
                            let _ = this.update_in(cx, move |app, window, cx| match loaded {
                                Ok(loaded) => app.apply_loaded(target, loaded, window, cx),
                                Err(error) => app.set_file_error(error, cx),
                            });
                        }
                    }
                    Ok(None) => {}
                    Err(error) => {
                        let _ = this.update(cx, |app, cx| {
                            app.set_file_error(
                                format!("Could not open the file picker: {error}"),
                                cx,
                            )
                        });
                    }
                }
            })
            .detach();
        }

        #[cfg(target_family = "wasm")]
        {
            if let Err(error) = file_io::web::open_picker(target.id()) {
                self.set_file_error(error, cx);
            }
        }
    }

    /// Load text supplied by the web file bridge.
    #[cfg_attr(not(target_family = "wasm"), allow(dead_code))]
    pub(crate) fn load_text_named(
        &mut self,
        target: FileTarget,
        name: String,
        text: String,
        window: &mut Window,
        cx: &mut Context<'_, Self>,
    ) {
        match LoadedText::from_text(&name, text) {
            Ok(loaded) => self.apply_loaded(target, loaded, window, cx),
            Err(error) => self.set_file_error(error, cx),
        }
    }

    fn apply_loaded(
        &mut self,
        target: FileTarget,
        loaded: LoadedText,
        window: &mut Window,
        cx: &mut Context<'_, Self>,
    ) {
        let LoadedText { name, text, bytes } = loaded;
        let state = match target {
            FileTarget::Original => self.original.clone(),
            FileTarget::Modified => self.modified.clone(),
        };
        state.update(cx, |state, cx| state.set_value(text, window, cx));

        let info = FileInfo { name, bytes };
        match target {
            FileTarget::Original => self.original_file = Some(info),
            FileTarget::Modified => self.modified_file = Some(info),
        }

        self.file_error = None;
        self.diff = DiffResult::default();
        self.mode = Mode::Edit;
        self.mobile_pane = match target {
            FileTarget::Original => MobilePane::Original,
            FileTarget::Modified => MobilePane::Modified,
        };
        cx.notify();
    }

    pub(crate) fn set_file_error(
        &mut self,
        message: impl Into<SharedString>,
        cx: &mut Context<'_, Self>,
    ) {
        self.file_error = Some(message.into());
        cx.notify();
    }

    #[cfg(not(target_family = "wasm"))]
    fn load_dropped(
        &mut self,
        target: FileTarget,
        paths: &ExternalPaths,
        window: &mut Window,
        cx: &mut Context<'_, Self>,
    ) {
        let Some(path) = paths.paths().first() else {
            self.set_file_error("The drop did not contain a file.", cx);
            return;
        };

        match file_io::read_text_file(path) {
            Ok(loaded) => self.apply_loaded(target, loaded, window, cx),
            Err(error) => self.set_file_error(error, cx),
        }
    }

    fn line_count(&self, state: &Entity<TextareaState>, cx: &App) -> usize {
        state.read(cx).value().lines().count()
    }

    fn edit_view(&self, _window: &mut Window, cx: &mut Context<'_, Self>, narrow: bool) -> AnyElement {
        let original_lines = self.line_count(&self.original, cx);
        let modified_lines = self.line_count(&self.modified, cx);
        let can_compare = !self.original.read(cx).value().is_empty()
            || !self.modified.read(cx).value().is_empty();

        let panes = if narrow {
            let selected = match self.mobile_pane {
                MobilePane::Original => editor_card(
                    FileTarget::Original,
                    self.original.clone(),
                    original_lines,
                    true,
                    self.original_file.as_ref(),
                    cx,
                ),
                MobilePane::Modified => editor_card(
                    FileTarget::Modified,
                    self.modified.clone(),
                    modified_lines,
                    true,
                    self.modified_file.as_ref(),
                    cx,
                ),
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
                    FileTarget::Original,
                    self.original.clone(),
                    original_lines,
                    false,
                    self.original_file.as_ref(),
                    cx,
                ))
                .child(editor_card(
                    FileTarget::Modified,
                    self.modified.clone(),
                    modified_lines,
                    false,
                    self.modified_file.as_ref(),
                    cx,
                ))
        };

        let compare = Button::new("compare")
            .primary()
            .label("Compare")
            .disabled(!can_compare)
            .on_click(cx.listener(|this, _, window, cx| this.compare(window, cx)));
        let example = Button::new("example")
            .ghost()
            .label("Load example")
            .on_click(cx.listener(|this, _, window, cx| this.load_example(window, cx)));
        let swap = Button::new("swap")
            .ghost()
            .icon(IconName::ArrowLeftRight)
            .label("Swap")
            .on_click(cx.listener(|this, _, window, cx| this.swap(window, cx)));
        let clear = Button::new("clear")
            .danger()
            .icon(IconName::Trash)
            .label("Clear all")
            .disabled(!can_compare)
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
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(swap)
                        .child(clear),
                )
        };

        let file_error = self.file_error.clone();
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
            .when_some(file_error, |this, message| {
                this.child(file_error_banner(&message))
            })
            .child(actions);

        content.into_any_element()
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
                        .when(selected, |this| {
                            this.font_weight(FontWeight::MEDIUM)
                        })
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

    fn diff_view(&self, _window: &mut Window, cx: &mut Context<'_, Self>, narrow: bool) -> AnyElement {
        let theme = cx.theme();
        let stats = self.diff.stats;
        let rows = &self.diff.rows;

        let body = if narrow {
            unified_rows(rows, cx)
        } else {
            split_rows(rows, cx)
        };

        let diff_header = if narrow {
            div()
                .flex()
                .items_center()
                .h(px(36.))
                .px(px(14.))
                .border_b_1()
                .border_color(theme.border)
                .text_xs()
                .text_color(theme.muted_foreground)
                .child("Unified diff")
                .into_any_element()
        } else {
            div()
                .flex()
                .items_center()
                .h(px(40.))
                .border_b_1()
                .border_color(theme.border)
                .child(panel_header("Original", cx))
                .child(div().w(px(1.)).h_full().bg(theme.border))
                .child(panel_header("Modified", cx))
                .into_any_element()
        };

        div()
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .px(if narrow { px(16.) } else { px(28.) })
            .pt(px(20.))
            .pb(if narrow { px(12.) } else { px(18.) })
            .gap_3()
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
                                Button::new("back")
                                    .ghost()
                                    .label("← Edit")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.back(window, cx)
                                    })),
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
                                            .text_color(theme.muted_foreground)
                                            .child(format!("{} rows", rows.len())),
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
                    .flex_col()
                    .flex_1()
                    .min_h_0()
                    .rounded(px(10.))
                    .border_1()
                    .border_color(theme.border)
                    .bg(theme.group_box)
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
                            .child(body),
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

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(background)
            .text_color(foreground)
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
            div()
                .flex()
                .items_center()
                .gap_3()
                .child(
                    div()
                        .text_lg()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child("Text Diff"),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_4()
                .child(
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
    target: FileTarget,
    state: Entity<TextareaState>,
    lines: usize,
    narrow: bool,
    file: Option<&FileInfo>,
    cx: &mut Context<'_, DiffApp>,
) -> Div {
    let border = cx.theme().border;
    let group_box = cx.theme().group_box;
    let muted_foreground = cx.theme().muted_foreground;

    let card = div()
        .flex()
        .flex_col()
        .w_full()
        .min_w_0()
        .min_h_0()
        .rounded(px(10.))
        .border_1()
        .border_color(border)
        .bg(group_box)
        .overflow_hidden()
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .h(px(44.))
                .px(px(16.))
                .border_b_1()
                .border_color(border)
                .child(
                    div()
                        .text_xs()
                        .text_color(muted_foreground)
                        .child(target.label()),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .when_some(file, |this, file| {
                            this.child(
                                div()
                                    .max_w(if narrow { px(140.) } else { px(220.) })
                                    .overflow_hidden()
                                    .whitespace_nowrap()
                                    .text_size(px(11.))
                                    .text_color(muted_foreground)
                                    .child(format!(
                                        "{} · {}",
                                        file.name,
                                        file_io::format_size(file.bytes)
                                    )),
                            )
                        })
                        .when(!narrow, |this| {
                            this.child(
                                div()
                                    .text_size(px(11.))
                                    .text_color(muted_foreground)
                                    .child(format!("{lines} lines")),
                            )
                        })
                        .child(
                            Button::new(target.button_id())
                                .ghost()
                                .label("Open")
                                .on_click(
                                    cx.listener(move |this, _, _, cx| this.open_file(target, cx)),
                                ),
                        ),
                ),
        )
        .child(
            div()
                .flex_1()
                .min_h_0()
                .p(px(12.))
                .child(
                    Textarea::new(&state)
                        .w_full()
                        .h(relative(1.))
                        .appearance(false)
                        .bordered(false)
                        .aria_label(target.label()),
                ),
        );

    #[cfg(not(target_family = "wasm"))]
    let card = card.on_drop(cx.listener(move |this, paths: &ExternalPaths, window, cx| {
        this.load_dropped(target, paths, window, cx);
    }));

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

fn file_error_banner(message: &str) -> Div {
    let red = theme::color(theme::RED);
    let red_bg = theme::color(theme::RED_BG);
    div()
        .flex()
        .items_center()
        .rounded(px(8.))
        .border_1()
        .border_color(red)
        .bg(red_bg)
        .px(px(12.))
        .py(px(8.))
        .text_sm()
        .text_color(red)
        .child(message.to_string())
}

fn empty_state(cx: &App) -> Div {
    let muted = cx.theme().muted_foreground;
    div()
        .flex()
        .items_center()
        .justify_center()
        .h(px(120.))
        .text_sm()
        .text_color(muted)
        .child("The two texts are identical.")
}

fn split_rows(rows: &[DiffRow], cx: &App) -> AnyElement {
    div()
        .w_full()
        .min_w(px(980.))
        .when(rows.is_empty(), |this| this.child(empty_state(cx)))
        .children(rows.iter().map(|row| diff_row(row, cx)))
        .into_any_element()
}

fn unified_rows(rows: &[DiffRow], cx: &App) -> AnyElement {
    let mut lines = Vec::new();
    for row in rows {
        if row.left.kind != SideKind::Empty {
            lines.push(unified_row(&row.left, cx).into_any_element());
        }
        if row.right.kind != SideKind::Empty {
            lines.push(unified_row(&row.right, cx).into_any_element());
        }
    }

    div()
        .flex()
        .flex_col()
        .w_full()
        .when(rows.is_empty(), |this| this.child(empty_state(cx)))
        .children(lines)
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

    let mut row = div().flex().w_full().min_h(px(22.));
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
            .child(side.number.map(|number| number.to_string()).unwrap_or_default()),
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
        .w_full()
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
        SideKind::Empty => (Some(theme::color(theme::PANEL_ALT)), None, theme.muted_foreground),
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
                .child(side.number.map(|number| number.to_string()).unwrap_or_default()),
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
                .when(!side.text.is_empty(), |this| {
                    this.child(styled_line(side))
                }),
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
