use eframe::egui;

pub const BACKGROUND: egui::Color32 = egui::Color32::from_rgb(3, 15, 11);
pub const PANEL: egui::Color32 = egui::Color32::from_rgb(6, 24, 17);
pub const GREEN: egui::Color32 = egui::Color32::from_rgb(85, 255, 151);
pub const DIM: egui::Color32 = egui::Color32::from_rgb(43, 126, 78);

pub fn install(ctx: &egui::Context) {
    let mut style = (*ctx.global_style()).clone();
    for font in style.text_styles.values_mut() {
        font.family = egui::FontFamily::Monospace;
    }
    let mut visuals = egui::Visuals::dark();
    visuals.override_text_color = Some(GREEN);
    visuals.panel_fill = BACKGROUND;
    visuals.window_fill = PANEL;
    visuals.extreme_bg_color = BACKGROUND;
    visuals.faint_bg_color = PANEL;
    visuals.selection.bg_fill = DIM;
    visuals.selection.stroke = egui::Stroke::new(1.0, GREEN);
    visuals.hyperlink_color = GREEN;
    for widget in [
        &mut visuals.widgets.noninteractive,
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        widget.corner_radius = egui::CornerRadius::ZERO;
        widget.bg_fill = PANEL;
        widget.weak_bg_fill = PANEL;
        widget.bg_stroke = egui::Stroke::new(1.0, DIM);
        widget.fg_stroke = egui::Stroke::new(1.0, GREEN);
    }
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(12, 55, 33);
    visuals.widgets.hovered.weak_bg_fill = visuals.widgets.hovered.bg_fill;
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, GREEN);
    visuals.widgets.active.bg_fill = DIM;
    visuals.widgets.active.weak_bg_fill = DIM;
    style.visuals = visuals;
    ctx.set_global_style(style);
}

/// A static, low-opacity overlay needs no animation or extra repaint loop.
pub fn scanlines(ui: &egui::Ui) {
    let rect = ui.max_rect();
    let painter = ui
        .ctx()
        .layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::new("crt_scanlines"),
        ))
        .with_clip_rect(rect);
    let mut y = rect.top();
    while y < rect.bottom() {
        painter.line_segment(
            [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
            egui::Stroke::new(1.0, egui::Color32::from_black_alpha(20)),
        );
        y += 4.0;
    }
}
