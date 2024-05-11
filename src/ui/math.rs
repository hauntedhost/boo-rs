use super::format::Displayable;
use ratatui::prelude::*;

pub fn area_height_minus_border(area: Rect) -> u16 {
    area.height.checked_sub(2).unwrap_or(0)
}

pub fn area_width_minus_border(area: Rect) -> u16 {
    area.width.checked_sub(2).unwrap_or(0)
}

pub fn get_wrapped_line_counts<T>(area: Rect, items: &[T]) -> Vec<usize>
where
    T: Displayable,
{
    let width = area_width_minus_border(area) as usize;
    let mut line_counts = vec![];
    for item in items.iter() {
        let text = item.display().to_string();
        let wrapped_texts = textwrap::wrap(&text, width);
        line_counts.push(wrapped_texts.len());
    }
    line_counts
}
