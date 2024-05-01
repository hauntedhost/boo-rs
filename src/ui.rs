use ratatui::{prelude::*, widgets::*};

use crate::app::{AppState, Onboarding, Sidebar};

/// This module contains all code for rendering the UI within the main app loop.

#[allow(unused_variables)]
pub fn render(frame: &mut Frame, app: &AppState) {
    let AppState {
        user,
        input,
        messages,
        users,
        ..
    } = app;
    let (rooms_width, messages_width, sidebar_width) = match app.sidebar {
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

    // dim style for widgets if user has not finished onboarding
    let widget_style = if app.onboarding == Onboarding::Completed {
        Style::default()
    } else {
        Style::new().dim()
    };

    // rooms area
    // TODO: bold the current room row
    let rooms = app.get_rooms_with_counts();
    let rooms_widget = build_rooms_widget(app.room.clone(), &rooms).style(widget_style);
    frame.render_widget(rooms_widget, rooms_area);

    // messages area
    let messages_widget =
        build_messages_widget(messages_area, app.room.clone(), messages).style(widget_style);
    frame.render_widget(messages_widget, messages_area);

    // sidebar area
    match app.sidebar {
        Sidebar::Users => {
            let users = app.get_uuid_username_pairs();
            let users_widget =
                build_users_widget(app.user.uuid.clone(), &users).style(widget_style);
            frame.render_widget(users_widget, sidebar_area);
        }
        Sidebar::Logs => {
            let logs = app.get_logs();
            let logs_widget = build_logs_widget(sidebar_area, &logs).style(widget_style);
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

fn build_rooms_widget(app_room_name: String, rooms: &Vec<(String, u32)>) -> Table {
    let mut rows: Vec<Row> = vec![];
    for (room_name, user_count) in rooms {
        let mut room_name = format!("{room_name}");
        if room_name == app_room_name {
            room_name.insert_str(0, "* ");
        }
        let row = Row::new(vec![format!("{room_name}"), format!("{user_count}")]);
        rows.push(row);
    }

    let widths = [Constraint::Fill(1), Constraint::Min(1)];
    let table = Table::new(rows, widths)
        .column_spacing(1)
        .flex(layout::Flex::Legacy)
        .block(Block::default().borders(Borders::ALL).title(" Rooms "));

    table
}

fn build_messages_widget(area: Rect, room: String, messages: &Vec<String>) -> List {
    let list_items = build_list_items(area, messages, 2, default_formatter);

    // pad items to push chat to the bottom
    let padded_items = pad_list(list_items.clone(), area.height - 2);

    let title = format!(" Chat room: {room} ");
    let message_list = List::new(padded_items)
        .direction(ListDirection::TopToBottom)
        .block(Block::default().borders(Borders::ALL).title(title));
    message_list
}

fn build_users_widget(app_user_uuid: String, users: &Vec<(String, String)>) -> Table {
    let mut rows: Vec<Row> = vec![];
    for (uuid, username) in users {
        let mut username = format!("{username}");
        if uuid.clone() == app_user_uuid {
            username.insert_str(0, "* ");
        }
        let row = Row::new(vec![username]);
        rows.push(row);
    }
    let widths = [Constraint::Fill(1)];
    let table = Table::new(rows, widths)
        .column_spacing(1)
        .flex(layout::Flex::Legacy)
        .block(Block::default().borders(Borders::ALL).title(" Users "));

    table
}

#[allow(dead_code)]
fn build_logs_widget(area: Rect, logs: &Vec<String>) -> List {
    let log_items = build_list_items(area, logs, 2, json_formatter);
    let log_list = List::new(log_items)
        .direction(ListDirection::TopToBottom)
        .block(Block::default().borders(Borders::ALL).title(" Logs "));
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
            // dim the input if input is still the generated username
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
                let text = format!(" Message #{}", app.room);
                let span = Span::raw(text).style(Style::new().italic().dim());
                Paragraph::new(span)
            } else {
                input_width = app.input.len() as u16;
                Paragraph::new(app.input.clone())
            }
        }
    };

    let input_block = input_paragraph.block(Block::default().borders(Borders::ALL));

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
