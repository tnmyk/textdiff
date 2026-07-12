use eframe::egui::{self, Color32, CornerRadius, Stroke, style::{WidgetVisuals, Widgets}};

pub fn apply_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    
    // Sleek dark mode palette
    let bg_color = Color32::from_rgb(18, 18, 20);
    let panel_bg = Color32::from_rgb(24, 24, 28);
    let widget_bg = Color32::from_rgb(32, 32, 38);
    let hover_bg = Color32::from_rgb(45, 45, 55);
    let accent = Color32::from_rgb(90, 100, 255); // Indigo blue accent
    let text_color = Color32::from_rgb(230, 230, 240);

    visuals.window_fill = bg_color;
    visuals.panel_fill = bg_color;
    visuals.extreme_bg_color = panel_bg; // Input boxes & scroll areas
    visuals.faint_bg_color = widget_bg;
    
    visuals.override_text_color = Some(text_color);

    let corner_radius = CornerRadius::same(6); // Modern soft corners

    // Configure widgets
    let mut widgets = Widgets::default();
    
    widgets.noninteractive = WidgetVisuals {
        bg_fill: Color32::TRANSPARENT,
        weak_bg_fill: Color32::TRANSPARENT,
        bg_stroke: Stroke::new(1.0, Color32::from_rgb(40, 40, 50)),
        corner_radius,
        fg_stroke: Stroke::new(1.0, text_color),
        expansion: 0.0,
    };
    
    widgets.inactive = WidgetVisuals {
        bg_fill: widget_bg,
        weak_bg_fill: widget_bg,
        bg_stroke: Stroke::new(1.0, Color32::from_rgb(45, 45, 55)),
        corner_radius,
        fg_stroke: Stroke::new(1.0, text_color),
        expansion: 0.0,
    };
    
    widgets.hovered = WidgetVisuals {
        bg_fill: hover_bg,
        weak_bg_fill: hover_bg,
        bg_stroke: Stroke::new(1.0, accent),
        corner_radius,
        fg_stroke: Stroke::new(1.0, Color32::WHITE),
        expansion: 1.0,
    };
    
    widgets.active = WidgetVisuals {
        bg_fill: accent,
        weak_bg_fill: accent,
        bg_stroke: Stroke::new(1.0, accent),
        corner_radius,
        fg_stroke: Stroke::new(1.0, Color32::WHITE),
        expansion: 1.0,
    };

    visuals.widgets = widgets;
    visuals.selection.bg_fill = accent.linear_multiply(0.4);
    visuals.selection.stroke = Stroke::new(1.0, accent);

    ctx.set_visuals(visuals);

    // Add luxurious padding and spacing
    let mut style = eframe::egui::Style::default();
    style.spacing.item_spacing = eframe::egui::vec2(12.0, 12.0);
    style.spacing.window_margin = eframe::egui::Margin::same(16);
    style.spacing.button_padding = eframe::egui::vec2(20.0, 10.0); // Thick, premium buttons
    
    // Make text slightly larger and more readable globally
    for font_id in style.text_styles.values_mut() {
        font_id.size *= 1.1; 
    }
    
    ctx.set_style_of(eframe::egui::Theme::Dark, style.clone());
    ctx.set_style_of(eframe::egui::Theme::Light, style);
}
