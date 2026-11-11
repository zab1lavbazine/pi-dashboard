use eframe::egui;

#[warn(dead_code)]
pub struct LayoutSizes {
    pub button_width: f32,
    pub button_height: f32,
    pub spacing: f32,
}

pub fn calculate_layout(ui: &egui::Ui) -> LayoutSizes {
    let width = ui.available_width();
    let height = ui.available_height();

    let spacing = 20.0;
    let columns = 3.0;

    let button_width = (width - spacing * (columns - 1.0)) / columns;

    let button_height = height * 0.18;

    LayoutSizes {
        button_width,
        button_height,
        spacing,
    }
}
