use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render_scrollbar(
    frame: &mut Frame,
    area: Rect,
    area_height: usize,
    line_length: usize,
    scrollbar_position: usize,
) {
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"))
        .track_symbol(Some("│"));

    // conditionally show scrollbar
    // scrollbar is only visible if content_length > 0
    let content_length = if line_length >= area_height {
        line_length.checked_sub(area_height).unwrap_or(0)
    } else {
        0
    };

    let mut scrollbar_state = ScrollbarState::default()
        .content_length(content_length)
        // .viewport_content_length(1)
        .position(scrollbar_position);

    frame.render_stateful_widget(
        scrollbar,
        area.inner(&Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut scrollbar_state,
    );
}
