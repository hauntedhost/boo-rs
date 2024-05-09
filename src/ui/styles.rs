use ratatui::prelude::*;

pub fn get_title_style() -> Style {
    Style::new().light_blue().not_dim()
}

pub fn get_selection_style(a: &String, b: &String) -> Style {
    if a == b {
        Style::new().not_dim()
    } else {
        Style::new().dim()
    }
}
