use gpui_kit::component::theme::{Theme, ThemeMode};
use gpui_kit::{App, Hsla, px, rgb};

pub const BG: u32 = 0x0A0E13;
pub const PANEL: u32 = 0x10161F;
pub const PANEL_ALT: u32 = 0x0D131B;
pub const BORDER: u32 = 0x1E2936;
pub const TEXT: u32 = 0xE6EDF6;
pub const MUTED: u32 = 0x7D8CA1;
pub const ACCENT: u32 = 0xF5B761;
pub const ACCENT_FG: u32 = 0x1A1206;
pub const LINK: u32 = 0x8FB4FF;
pub const RED: u32 = 0xF87171;
pub const RED_BG: u32 = 0x3A1D22;
pub const RED_STRONG: u32 = 0x8E343C;
pub const GREEN: u32 = 0x4ADE80;
pub const GREEN_BG: u32 = 0x123326;
pub const GREEN_STRONG: u32 = 0x1F6B47;

pub fn apply(cx: &mut App) {
    Theme::change(ThemeMode::Dark, None, cx);
    Theme::update(cx, |theme| {
        theme.font_family = "Inter".into();
        theme.mono_font_family = "JetBrains Mono".into();
        theme.font_size = px(14.);
        theme.mono_font_size = px(13.);
        theme.radius = px(8.);
        theme.radius_lg = px(14.);

        theme.background = color(BG);
        theme.foreground = color(TEXT);
        theme.border = color(BORDER);
        theme.input = color(PANEL);
        theme.muted = color(PANEL_ALT);
        theme.muted_foreground = color(MUTED);
        theme.primary = color(ACCENT);
        theme.primary_hover = color(0xFFC878);
        theme.primary_active = color(0xE5A64F);
        theme.primary_foreground = color(ACCENT_FG);
        theme.button_primary = color(ACCENT);
        theme.button_primary_hover = color(0xFFC878);
        theme.button_primary_active = color(0xE5A64F);
        theme.button_primary_foreground = color(ACCENT_FG);
        theme.secondary = color(PANEL_ALT);
        theme.secondary_hover = color(0x18212C);
        theme.secondary_active = color(BORDER);
        theme.secondary_foreground = color(TEXT);
        theme.button_secondary = color(PANEL_ALT);
        theme.button_secondary_hover = color(0x18212C);
        theme.button_secondary_active = color(BORDER);
        theme.button_secondary_foreground = color(TEXT);
        theme.accent = color(PANEL);
        theme.accent_foreground = color(TEXT);
        theme.link = color(LINK);
        theme.link_hover = color(0xB3CBFF);
        theme.link_active = color(0x6E97F0);
        theme.group_box = color(PANEL);
        theme.group_box_foreground = color(TEXT);
        theme.caret = color(ACCENT);
        theme.selection = color(ACCENT).opacity(0.28);
        theme.title_bar = color(BG);
        theme.title_bar_border = color(BORDER);
        theme.sidebar = color(PANEL_ALT);
        theme.sidebar_border = color(BORDER);
        theme.sidebar_foreground = color(TEXT);
        theme.sidebar_accent = color(PANEL);
        theme.sidebar_accent_foreground = color(TEXT);
        theme.success = color(GREEN);
        theme.success_foreground = color(0x05140B);
        theme.danger = color(RED);
        theme.danger_foreground = color(0x1A0505);
    });
}

pub fn color(value: u32) -> Hsla {
    rgb(value).into()
}
