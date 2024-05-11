use crate::app::AppState;
use crate::app::Onboarding;
use crate::app::SocketStatus;
use crate::ui::symbols::*;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render_widget(frame: &mut Frame, area: Rect, app: &AppState) {
    let widget = build_widget(
        app.input.clone(),
        app.get_username(),
        app.room.clone(),
        app.onboarding,
        app.socket_url.clone(),
        app.socket_status.clone(),
        app.is_socket_active(),
    );
    frame.render_widget(widget, area);
}

fn build_widget(
    input: String,
    username: String,
    room: String,
    onboarding: Onboarding,
    socket_url: Option<String>,
    socket_status: SocketStatus,
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

    let socket_symbol = if is_socket_active {
        SOCKET_ACTIVE_SYMBOL
    } else {
        SOCKET_STATUS_SYMBOL
    };

    let socket_style = match socket_status {
        SocketStatus::Closed => Style::default().dim(),
        SocketStatus::Connected => Style::default().light_blue().bold(),
        SocketStatus::ConnectFailed => Style::default().light_red().not_dim(),
        SocketStatus::Disconnected => Style::default().light_red().not_dim(),
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
                Span::styled(format!("{socket_symbol} "), socket_style),
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
