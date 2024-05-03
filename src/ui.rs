use ratatui::{prelude::*, widgets::*};

use crate::app::{AppState, Focus, Onboarding, Sidebar};

/// This module contains all code for rendering the UI within the main app loop.

#[allow(unused_variables)]
pub fn render(frame: &mut Frame, app: &mut AppState) {
    let AppState { user, input, .. } = app;

    if app.should_show_help() {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(100)])
            .split(frame.size());

        let help_area = layout[0];

        let items = vec![
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
        let available_padding_x = help_area.width.checked_sub(max_line_length).unwrap_or(0);
        let padding_x = if available_padding_x >= 2 {
            available_padding_x / 2
        } else {
            0
        };

        let help_widget = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Help ")
                .title_alignment(Alignment::Center)
                .padding(Padding::symmetric(padding_x, 1)),
        );

        frame.render_widget(help_widget, help_area);

        return ();
    }

    let (rooms_width, messages_width, sidebar_width) = match app.ui_sidebar_view {
        Sidebar::Users => (25, 55, 20),
        Sidebar::Logs => (25, 40, 35),
    };

    let outer_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Percentage(rooms_width),
            Constraint::Percentage(messages_width),
            Constraint::Percentage(sidebar_width),
        ])
        .split(frame.size());

    let (rooms_area, messages_layout, sidebar_area) =
        (outer_layout[0], outer_layout[1], outer_layout[2]);

    let inner_layout = Layout::default()
        .direction(Direction::Vertical)
        .spacing(0)
        .margin(0)
        .constraints(vec![Constraint::Min(1), Constraint::Length(3)])
        .split(messages_layout);
    let (messages_area, input_area) = (inner_layout[0], inner_layout[1]);

    // rooms area
    let rooms = app.get_rooms_with_counts();
    let rooms_widget =
        build_rooms_widget(&rooms, app.room.clone(), app.onboarding, app.ui_focus_area);
    app.ui_room_table_state
        .select(app.get_selected_or_current_room_index());
    frame.render_stateful_widget(rooms_widget, rooms_area, &mut app.ui_room_table_state);

    // messages area
    let messages = app.get_messages();
    let messages_widget =
        build_messages_widget(messages_area, app.room.clone(), &messages, app.onboarding);
    frame.render_widget(messages_widget, messages_area);

    // sidebar area
    match app.ui_sidebar_view {
        Sidebar::Users => {
            let users = app.get_uuid_username_pairs();
            let users_widget = build_users_widget(app.user.uuid.clone(), &users, app.onboarding);
            frame.render_widget(users_widget, sidebar_area);
        }
        Sidebar::Logs => {
            let logs = app.get_logs();
            let logs_widget = build_logs_widget(sidebar_area, &logs, app.onboarding);
            frame.render_widget(logs_widget, sidebar_area);
        }
    };

    // input area
    let (input_widget, input_width) = build_input_widget(&app);
    frame.render_widget(input_widget, input_area);

    // cursor
    let x = input_area.x + 1 + input_width;
    let y = input_area.y + 1;
    frame.set_cursor(x, y);
}

fn build_rooms_widget(
    rooms: &Vec<(String, u32)>,
    current_room: String,
    onboarding: Onboarding,
    focus: Focus,
) -> Table {
    let mut rows: Vec<Row> = vec![];
    for (room_name, user_count) in rooms {
        let room_name = format!("{room_name}");

        let row_style = if room_name == current_room {
            Style::new().light_green()
        } else {
            Style::default()
        };

        let row = Row::new(vec![format!("{room_name}"), format!("{user_count}")]).style(row_style);
        rows.push(row);
    }

    let widths = [Constraint::Fill(1), Constraint::Min(1)];

    let (border_style, title_style) = if onboarding == Onboarding::Completed {
        if focus == Focus::Rooms {
            (Style::new().not_dim(), Style::new().not_dim())
        } else {
            (Style::new().dim(), Style::new().dim())
        }
    } else {
        (Style::new().dim(), Style::new().dim())
    };

    let table = Table::new(rows, widths)
        .column_spacing(1)
        .flex(layout::Flex::Legacy)
        .highlight_symbol("> ")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(" Rooms ")
                .title_style(title_style),
        );

    table
}

fn build_messages_widget(
    area: Rect,
    room: String,
    messages: &Vec<String>,
    onboarding: Onboarding,
) -> List {
    let list_items = build_list_items(area, messages, 2, default_formatter);

    // pad items to push chat to the bottom
    let padded_items = pad_list(list_items.clone(), area.height - 2);

    let title = format!(" Chat room: {room} ");

    // dim title if user has not finished onboarding
    let title_style = if onboarding == Onboarding::Completed {
        Style::new().not_dim()
    } else {
        Style::new().dim()
    };

    let message_list = List::new(padded_items)
        .direction(ListDirection::TopToBottom)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::new().dim())
                .title(title)
                .title_style(title_style),
        );
    message_list
}

fn build_users_widget(
    app_user_uuid: String,
    users: &Vec<(String, String)>,
    onboarding: Onboarding,
) -> Table {
    let mut rows: Vec<Row> = vec![];
    for (uuid, username) in users {
        let username = format!("{username}");
        let style = if uuid.clone() == app_user_uuid {
            Style::new().light_green()
        } else {
            Style::default()
        };
        let row = Row::new(vec![username]).style(style);
        rows.push(row);
    }
    let widths = [Constraint::Fill(1)];

    // dim title if user has not finished onboarding
    let title_style = if onboarding == Onboarding::Completed {
        Style::new().not_dim()
    } else {
        Style::new().dim()
    };

    let table = Table::new(rows, widths)
        .column_spacing(1)
        .flex(layout::Flex::Legacy)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::new().dim())
                .title(" Users ")
                .title_style(title_style),
        );

    table
}

#[allow(dead_code)]
fn build_logs_widget(area: Rect, logs: &Vec<String>, onboarding: Onboarding) -> List {
    let log_items = build_list_items(area, logs, 2, json_formatter);

    // dim title if user has not finished onboarding
    let title_style = if onboarding == Onboarding::Completed {
        Style::new().not_dim()
    } else {
        Style::new().dim()
    };

    let log_list = List::new(log_items)
        .direction(ListDirection::TopToBottom)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::new().dim())
                .title(" Logs ")
                .title_style(title_style),
        );
    log_list
}

fn build_input_widget(app: &AppState) -> (Paragraph, u16) {
    let input_width: u16;

    let input_paragraph = match app.onboarding {
        Onboarding::ConfirmingRoomName => {
            // dim the input if room is still the generated room name
            let style = if app.input == app.room {
                Style::new().italic().dim()
            } else {
                Style::default()
            };

            let line = Line::from(vec![
                Span::raw("Enter a room name > "),
                Span::styled(app.input.clone(), style),
            ]);
            input_width = line.width() as u16;
            Paragraph::new(line)
        }
        Onboarding::ConfirmingUsername => {
            // dim the input text if input is still the generated username
            let style = if app.input == app.user.username {
                Style::new().italic().dim()
            } else {
                Style::default()
            };

            let line = Line::from(vec![
                Span::raw("Enter a username > "),
                Span::styled(app.input.clone(), style),
            ]);
            input_width = line.width() as u16;
            Paragraph::new(line)
        }
        Onboarding::Completed => {
            if app.input_is_blank() {
                input_width = 0;
                let text = format!("Message #{}", app.room);
                let span = Span::raw(text).style(Style::new().italic().dim());
                Paragraph::new(span)
            } else {
                input_width = app.input.len() as u16;
                Paragraph::new(app.input.clone())
            }
        }
    };

    let border_style = if app.ui_focus_area == Focus::Input {
        Style::new().not_dim()
    } else {
        Style::new().dim()
    };

    let input_block = input_paragraph.block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(border_style),
    );

    (input_block, input_width)
}

fn build_list_items<F>(
    area: Rect,
    all_items: &Vec<String>,
    padding: u16,
    formatter: F,
) -> Vec<ListItem>
where
    F: Fn(&String) -> String,
{
    let max_lines = (area.height - padding) as usize;

    let mut line_count = 0;
    let mut items: Vec<ListItem> = vec![];
    for item in all_items.iter().rev() {
        let formatted_item = formatter(&item);

        let width = (area.width - padding) as usize;
        let wrapped_texts = textwrap::wrap(&formatted_item, width);
        let lines = wrapped_texts
            .iter()
            .map(|s| Line::from(Span::raw(s.to_string())))
            .collect::<Vec<_>>();

        if line_count + lines.len() > max_lines {
            let lines_fit = max_lines - line_count;
            if lines_fit > 0 {
                // Only take as many lines as we can fit
                let mut sliced_lines = lines.into_iter().rev().take(lines_fit).collect::<Vec<_>>();
                sliced_lines.reverse();
                let list_item = ListItem::from(Text::from(sliced_lines));
                items.push(list_item);
            }
            break;
        } else {
            let list_item = ListItem::from(Text::from(lines.clone()));
            items.push(list_item);
            line_count += lines.len();
        }
    }

    items.reverse();
    items
}

fn default_formatter(string: &String) -> String {
    string.clone()
}

fn json_formatter(string: &String) -> String {
    let json: serde_json::Value = serde_json::from_str(string).unwrap();

    // example: filter presence_state events
    // if let serde_json::Value::Array(elements) = json.clone() {
    //     if let Some(event) = elements.get(3).and_then(|e| e.as_str()) {
    //         if event != "presence_state" {
    //             return "".to_string();
    //         }
    //     }
    // }

    let pretty_json = serde_json::to_string_pretty(&json).unwrap();
    let formatted_log = format!("=> {}", pretty_json.trim());

    formatted_log
}

fn pad_list(mut items: Vec<ListItem>, count: u16) -> Vec<ListItem> {
    let count = count as usize;

    while items.len() < count {
        items.insert(0, ListItem::from(Text::from("")));
    }

    items
}
