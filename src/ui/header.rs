use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::Onboarding;
use crate::ui::symbols::*;

pub fn build_widget(
    input: String,
    username: String,
    room: String,
    onboarding: Onboarding,
    socket_url: Option<String>,
    is_socket_active: bool,
) -> Table<'static> {
    let socket_url = socket_url.unwrap_or_else(|| "".to_string());

    let (username, sep, room) = match onboarding {
        Onboarding::ConfirmingUsername => {
            let username = if input == username { username } else { input };
            (username, "".to_string(), "".to_string())
        }
        Onboarding::ConfirmingRoom => {
            let room = if input == room { room } else { input };
            (username, format!(" {HASH_SYMBOL} "), room)
        }
        Onboarding::Completed => (username, format!(" {HASH_SYMBOL} "), room),
    };

    let socket_style = if is_socket_active {
        Style::default().light_blue().not_dim().slow_blink()
    } else {
        Style::default().light_blue()
    };

    let row = Row::new(vec![
        Cell::from(Line::from(vec![
            Span::styled("@ ", Style::default().light_blue().bold()),
            Span::raw(username),
        ])),
        Cell::from(
            Line::from(vec![
                Span::styled(sep, Style::default().light_blue().bold()),
                Span::raw(room),
            ])
            .alignment(Alignment::Center),
        ),
        Cell::from(
            Line::from(vec![
                Span::styled(format!("{SOCKET_SYMBOL} "), socket_style),
                Span::raw(socket_url),
            ])
            .alignment(Alignment::Right),
        ),
    ]);

    let rows = vec![row];
    let widths = [
        Constraint::Fill(1),
        Constraint::Fill(1),
        Constraint::Fill(1),
    ];

    let table = Table::new(rows, widths)
        .column_spacing(1)
        .flex(layout::Flex::Legacy)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::new().dim())
                .padding(Padding::symmetric(1, 0)),
        )
        .style(Style::new().dim());

    table
}
