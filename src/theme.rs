use gpui_kit::component::theme::{Theme, ThemeMode};
use gpui_kit::{App, Hsla, px, rgb};

pub const BG: u32 = 0x0B0C0E;
pub const PANEL: u32 = 0x121316;
pub const PANEL_ALT: u32 = 0x0E0F12;
pub const BORDER: u32 = 0x24262B;
pub const TEXT: u32 = 0xE8EAED;
pub const MUTED: u32 = 0x8B909A;
pub const ACCENT: u32 = 0xE8EAED;
pub const ACCENT_FG: u32 = 0x0B0C0E;
pub const LINK: u32 = 0xAEB4BE;
pub const RED: u32 = 0xD9777F;
pub const RED_BG: u32 = 0x261619;
pub const RED_STRONG: u32 = 0x6B3036;
pub const GREEN: u32 = 0x7FB58A;
pub const GREEN_BG: u32 = 0x14231A;
pub const GREEN_STRONG: u32 = 0x2F5B3B;

pub fn apply(cx: &mut App) {
    Theme::change(ThemeMode::Dark, None, cx);
    Theme::update(cx, |theme| {
        theme.font_family = "Inter".into();
        theme.mono_font_family = "JetBrains Mono".into();
        theme.font_size = px(14.);
        theme.mono_font_size = px(13.);
        theme.radius = px(6.);
        theme.radius_lg = px(10.);

        theme.background = color(BG);
        theme.foreground = color(TEXT);
        theme.border = color(BORDER);
        theme.input = color(PANEL);
        theme.muted = color(PANEL_ALT);
        theme.muted_foreground = color(MUTED);
        theme.primary = color(ACCENT);
        theme.primary_hover = color(0xFFFFFF);
        theme.primary_active = color(0xD5D8DD);
        theme.primary_foreground = color(ACCENT_FG);
        theme.button_primary = color(ACCENT);
        theme.button_primary_hover = color(0xFFFFFF);
        theme.button_primary_active = color(0xD5D8DD);
        theme.button_primary_foreground = color(ACCENT_FG);
        theme.secondary = color(PANEL_ALT);
        theme.secondary_hover = color(0x17191D);
        theme.secondary_active = color(BORDER);
        theme.secondary_foreground = color(TEXT);
        theme.button_secondary = color(PANEL_ALT);
        theme.button_secondary_hover = color(0x17191D);
        theme.button_secondary_active = color(BORDER);
        theme.button_secondary_foreground = color(TEXT);
        theme.accent = color(PANEL);
        theme.accent_foreground = color(TEXT);
        theme.link = color(LINK);
        theme.link_hover = color(TEXT);
        theme.link_active = color(0xC9CDD4);
        theme.group_box = color(PANEL);
        theme.group_box_foreground = color(TEXT);
        theme.caret = color(TEXT);
        theme.selection = color(0xFFFFFF).opacity(0.16);
        theme.title_bar = color(BG);
        theme.title_bar_border = color(BORDER);
        theme.sidebar = color(PANEL_ALT);
        theme.sidebar_border = color(BORDER);
        theme.sidebar_foreground = color(TEXT);
        theme.sidebar_accent = color(PANEL);
        theme.sidebar_accent_foreground = color(TEXT);
        theme.success = color(GREEN);
        theme.success_foreground = color(0x06130A);
        theme.danger = color(RED);
        theme.danger_foreground = color(0x160607);
    });
}

pub fn color(value: u32) -> Hsla {
    rgb(value).into()
}
