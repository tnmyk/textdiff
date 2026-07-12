use eframe::egui::{self, Color32, CornerRadius, Stroke, style::{WidgetVisuals, Widgets}};

pub fn apply_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    
    // IntelliJ Darcula palette
    let bg_color = Color32::from_rgb(60, 63, 65); // #3C3F41 Main UI background
    let panel_bg = Color32::from_rgb(49, 51, 53); // #313335 Secondary panels
    let widget_bg = Color32::from_rgb(76, 80, 82); // #4C5052 Buttons, inactive widgets
    let hover_bg = Color32::from_rgb(92, 97, 100); // #5C6164 Hovered widgets
    let editor_bg = Color32::from_rgb(43, 43, 43); // #2B2B2B Editor background
    let accent = Color32::from_rgb(74, 136, 199); // #4A88C7 Primary blue accent
    let text_color = Color32::from_rgb(169, 183, 198); // #A9B7C6 Main text
    let border_color = Color32::from_rgb(50, 50, 50); // #323232

    visuals.window_fill = bg_color;
    visuals.panel_fill = bg_color;
    visuals.extreme_bg_color = editor_bg; // Input boxes & scroll areas
    visuals.faint_bg_color = panel_bg;
    
    visuals.override_text_color = Some(text_color);

    let corner_radius = CornerRadius::same(3); // Sharp, professional corners

    // Configure widgets
    let mut widgets = Widgets::default();
    
    widgets.noninteractive = WidgetVisuals {
        bg_fill: Color32::TRANSPARENT,
        weak_bg_fill: Color32::TRANSPARENT,
        bg_stroke: Stroke::new(1.0, border_color),
        corner_radius,
        fg_stroke: Stroke::new(1.0, text_color),
        expansion: 0.0,
    };
    
    widgets.inactive = WidgetVisuals {
        bg_fill: widget_bg,
        weak_bg_fill: widget_bg,
        bg_stroke: Stroke::new(1.0, border_color),
        corner_radius,
        fg_stroke: Stroke::new(1.0, text_color),
        expansion: 0.0,
    };
    
    widgets.hovered = WidgetVisuals {
        bg_fill: hover_bg,
        weak_bg_fill: hover_bg,
        bg_stroke: Stroke::new(1.0, border_color),
        corner_radius,
        fg_stroke: Stroke::new(1.0, Color32::WHITE),
        expansion: 0.0, // IntelliJ buttons don't grow
    };
    
    widgets.active = WidgetVisuals {
        bg_fill: accent,
        weak_bg_fill: accent,
        bg_stroke: Stroke::new(1.0, accent),
        corner_radius,
        fg_stroke: Stroke::new(1.0, Color32::WHITE),
        expansion: 0.0,
    };

    visuals.widgets = widgets;
    visuals.selection.bg_fill = Color32::from_rgb(33, 66, 131); // #214283 text selection
    visuals.selection.stroke = Stroke::new(1.0, accent);

    ctx.set_visuals(visuals);

    // Add professional padding and spacing
    let mut style = eframe::egui::Style::default();
    style.spacing.item_spacing = eframe::egui::vec2(8.0, 8.0);
    style.spacing.window_margin = eframe::egui::Margin::same(12);
    style.spacing.button_padding = eframe::egui::vec2(14.0, 6.0); // Compact, standard buttons
    
    ctx.set_style_of(eframe::egui::Theme::Dark, style.clone());
    ctx.set_style_of(eframe::egui::Theme::Light, style);

    ctx.set_zoom_factor(1.375);
}
