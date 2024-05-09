use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn build_widget(area: Rect) -> List<'static> {
    let items = vec![
        "Welcome to the chat! 👻",
        "",
        "Keyboard Shortcuts",
        "  Esc: Quit the application",
        "  Tab: Cycle focus from input to rooms",
        "  Alt + h: Show this help message",
        "  Alt + s: Toggle right sidebar view",
        "",
        "Commands",
        "  /?: Show this help message",
        "  /help: Show this help message",
        "  /join: Join a room",
        "  /quit: Quit the application",
        "",
        "Press any key to close this help message",
    ];

    let max_line_length = (items.iter().map(|line| line.len()).max().unwrap_or(0) + 2) as u16;
    let available_padding_x = area.width.checked_sub(max_line_length).unwrap_or(0);
    let padding_x = if available_padding_x >= 2 {
        available_padding_x / 2
    } else {
        0
    };

    List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Help ")
            .title_alignment(Alignment::Center)
            .padding(Padding::symmetric(padding_x, 1)),
    )
}
