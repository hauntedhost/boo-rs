use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::Message;
use crate::app::{AppState, Focus, Onboarding, RightSidebar};

/// This module contains all code for rendering the UI within the main app loop.

// TODO: Retry emoji symbols after this bugfix: https://github.com/ratatui-org/ratatui/issues/1032
// const CHAT_SYMBOL: &str = "👻";
// const HASH_SYMBOL: &str = "ⵌ";
// const LOG_SYMBOL: &str = "▤";
// const SOCKET_SYMBOL: &str = "☰";
const CHAT_SYMBOL: &str = "";
const HASH_SYMBOL: &str = "#";
const SOCKET_SYMBOL: &str = "=";
const LOG_SYMBOL: &str = "=";

#[allow(unused_variables)]
pub fn render(frame: &mut Frame, app: &mut AppState) {
    let AppState { user, input, .. } = app;

    if app.showing_help() {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(100)])
            .split(frame.size());
        let help_area = layout[0];
        let help_widget = build_help_widget(help_area);
        frame.render_widget(help_widget, help_area);
        return;
    }

    // outer vertical layout
    // - info area
    // - main_outer_layout

    let outer_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Length(3), Constraint::Min(1)])
        .split(frame.size());

    let (info_area, main_outer_layout) = (outer_layout[0], outer_layout[1]);

    // horizontal layout inside main_outer_layout
    // - users | messages_outer_layout | rooms

    let (left_sidebar_width, messages_width, right_sidebar_width) = match app.ui_right_sidebar_view
    {
        RightSidebar::Rooms => (22, 56, 22),
        RightSidebar::Logs => (25, 40, 35),
    };

    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Percentage(left_sidebar_width),
            Constraint::Percentage(messages_width),
            Constraint::Percentage(right_sidebar_width),
        ])
        .split(main_outer_layout);

    let (left_sidebar_area, messages_outer_layout, right_sidebar_area) =
        (main_layout[0], main_layout[1], main_layout[2]);

    // vertical layout inside messages_outer_layout
    // - messages area
    // - input area

    let messages_layout = Layout::default()
        .direction(Direction::Vertical)
        .spacing(0)
        .margin(0)
        .constraints(vec![Constraint::Min(1), Constraint::Length(3)])
        .split(messages_outer_layout);

    let (messages_area, input_area) = (messages_layout[0], messages_layout[1]);

    // info area

    let info_widget = build_info_widget(
        app.input.clone(),
        app.get_username(),
        app.room.clone(),
        app.onboarding,
        app.socket_url.clone(),
        app.is_socket_active(),
    );
    frame.render_widget(info_widget, info_area);

    // users area
    let users = app.get_uuid_username_pairs();
    let users_widget = build_users_widget(app.user.uuid.clone(), &users);
    frame.render_widget(users_widget, left_sidebar_area);

    // messages area
    let messages = app.get_messages();
    let messages_widget = build_messages_widget(messages_area, app.room.clone(), &messages);
    frame.render_widget(messages_widget, messages_area);

    // right_sidebar_area
    match app.ui_right_sidebar_view {
        RightSidebar::Rooms => {
            let rooms = app.get_rooms_with_counts();
            let rooms_widget = build_rooms_widget(&rooms, app.room.clone(), app.ui_focus_area);
            let selected_room = app.get_selected_or_current_room_index();
            app.ui_room_table_state.select(selected_room);
            frame.render_stateful_widget(
                rooms_widget,
                right_sidebar_area,
                &mut app.ui_room_table_state,
            );
        }
        RightSidebar::Logs => {
            let logs = app.get_logs();
            let logs_widget = build_logs_widget(right_sidebar_area, &logs);
            frame.render_widget(logs_widget, right_sidebar_area);
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

// Widgets

fn build_help_widget(area: Rect) -> List<'static> {
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

fn build_info_widget(
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

fn build_rooms_widget(rooms: &Vec<(String, u32)>, current_room: String, focus: Focus) -> Table {
    let mut rows: Vec<Row> = vec![];
    for (room_name, user_count) in rooms {
        let room_name = format!("{room_name}");
        let style = get_selection_style(&room_name, &current_room);
        let row = Row::new(vec![format!("{room_name}"), format!("{user_count}")]).style(style);
        rows.push(row);
    }

    let border_style = if focus == Focus::Rooms {
        Style::new().not_dim()
    } else {
        Style::new().dim()
    };

    Table::new(rows, [Constraint::Fill(1), Constraint::Min(1)])
        .column_spacing(1)
        .flex(layout::Flex::Legacy)
        .highlight_symbol("> ")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(format!(" {HASH_SYMBOL} Rooms "))
                .title_style(get_title_style()),
        )
}

fn build_messages_widget(area: Rect, _room: String, messages: &Vec<Message>) -> List {
    let (list_items, line_count) = build_list_items(area, messages, 2, default_formatter);

    // padding to push chat to the bottom
    let top_padding = (area.height - 2)
        .checked_sub(line_count as u16)
        .unwrap_or(0) as u16;

    List::new(list_items)
        .direction(ListDirection::TopToBottom)
        .block(
            Block::default()
                .padding(Padding::new(0, 0, top_padding, 0))
                .borders(Borders::ALL)
                .border_style(Style::new().dim())
                .title(format!(" {CHAT_SYMBOL} Chat "))
                .title_style(get_title_style()),
        )
}

fn build_input_widget(app: &AppState) -> (Paragraph, u16) {
    let input_width: u16;

    let input_paragraph = match app.onboarding {
        Onboarding::ConfirmingUsername => {
            // dim the input text if input is still the generated username
            let style = if app.input == app.user.username {
                Style::new().italic().dim()
            } else {
                Style::default()
            };

            let line = Line::from(vec![
                Span::raw("Enter a username"),
                Span::styled(" @ ", Style::default().italic()),
                Span::styled(app.input.clone(), style),
            ]);
            input_width = line.width() as u16;
            Paragraph::new(line)
        }
        Onboarding::ConfirmingRoom => {
            // dim the input if room is still the generated room name
            let style = if app.input == app.room {
                Style::new().italic().dim()
            } else {
                Style::default()
            };

            let line = Line::from(vec![
                Span::raw("Enter a room name"),
                Span::styled(" # ", Style::default().italic()),
                Span::styled(app.input.clone(), style),
            ]);
            input_width = line.width() as u16;
            Paragraph::new(line)
        }
        Onboarding::Completed => {
            if app.input.is_empty() {
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

fn build_users_widget(app_user_uuid: String, users: &Vec<(String, String)>) -> Table {
    let mut rows: Vec<Row> = vec![];
    for (uuid, username) in users {
        let username = format!("{username}");
        let style = get_selection_style(&uuid, &app_user_uuid);
        let row = Row::new(vec![username]).style(style);
        rows.push(row);
    }

    Table::new(rows, [Constraint::Fill(1)])
        .column_spacing(1)
        .flex(layout::Flex::Legacy)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::new().dim())
                .title(" @ Users ")
                .title_style(get_title_style()),
        )
}

fn build_logs_widget(area: Rect, logs: &Vec<String>) -> List {
    let (log_items, _line_count) = build_list_items(area, logs, 2, json_formatter);
    let title_style = get_title_style();

    List::new(log_items)
        .direction(ListDirection::TopToBottom)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::new().dim())
                .title(format!(" {LOG_SYMBOL} Logs "))
                .title_style(title_style),
        )
}

// Styles

fn get_title_style() -> Style {
    Style::new().light_blue().not_dim()
}

fn get_selection_style(a: &String, b: &String) -> Style {
    if a == b {
        Style::new().not_dim()
    } else {
        Style::new().dim()
    }
}

// Helpers

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DisplayFormat {
    Plaintext,
    SystemMessage,
    UserMessage,
}

trait Displayable {
    fn display(&self) -> &str;
    fn format(&self) -> DisplayFormat;
}

impl Displayable for String {
    fn display(&self) -> &str {
        self
    }

    fn format(&self) -> DisplayFormat {
        DisplayFormat::Plaintext
    }
}

impl Displayable for Message {
    fn display(&self) -> &str {
        match self {
            Message::System(message) | Message::User(message) => message,
        }
    }

    fn format(&self) -> DisplayFormat {
        match self {
            Message::System(_) => DisplayFormat::SystemMessage,
            Message::User(_) => DisplayFormat::UserMessage,
        }
    }
}

fn build_list_items<T, F>(
    area: Rect,
    all_items: &[T],
    padding: u16,
    formatter: F,
) -> (Vec<ListItem>, usize)
where
    T: Displayable,
    F: Fn(&String) -> String,
{
    let max_lines = (area.height - padding) as usize;
    let mut line_count = 0;
    let mut items: Vec<ListItem> = vec![];
    for item in all_items.iter().rev() {
        let item_content = item.display().to_string();
        let item_style = match item.format() {
            DisplayFormat::Plaintext => Style::default(),
            DisplayFormat::SystemMessage => Style::default().italic().dim(),
            DisplayFormat::UserMessage => Style::default(),
        };
        let formatted_item = formatter(&item_content);

        // Wrap text to fit the area width
        let width = (area.width - padding) as usize;
        let wrapped_texts = textwrap::wrap(&formatted_item, width);
        let lines = if item.format() == DisplayFormat::UserMessage {
            // Highlight the username in user messages (very annoying code to handle text wrapping)
            let style = Style::default().light_green();
            let mut found_sep = false;
            let mut lines: Vec<ratatui::text::Line<'_>> = Vec::new();
            for text in wrapped_texts {
                if found_sep {
                    let line = Line::from(Span::raw(text.to_string()));
                    lines.push(line);
                    continue;
                }

                if let Some(index) = text.find(":") {
                    found_sep = true;
                    let username = text[..index].to_string();
                    let message = text[index..].to_string();
                    let line = Line::from(vec![Span::styled(username, style), Span::raw(message)]);
                    lines.push(line);
                } else {
                    let line = Line::from(Span::styled(text.to_string(), style));
                    lines.push(line);
                }
            }
            lines
        } else {
            wrapped_texts
                .iter()
                .map(|s| Line::from(Span::raw(s.to_string())))
                .collect::<Vec<_>>()
        };

        // Only take as many lines as we can fit
        if line_count + lines.len() > max_lines {
            let lines_fit = max_lines - line_count;
            if lines_fit > 0 {
                let mut sliced_lines = lines.into_iter().rev().take(lines_fit).collect::<Vec<_>>();
                sliced_lines.reverse();
                let list_item = ListItem::from(Text::from(sliced_lines)).style(item_style);
                items.push(list_item);
            }
            break;
        } else {
            let list_item = ListItem::from(Text::from(lines.clone())).style(item_style);
            items.push(list_item);
            line_count += lines.len();
        }
    }

    items.reverse();
    (items, line_count)
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
