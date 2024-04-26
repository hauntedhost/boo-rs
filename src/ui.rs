/// This module contains all code for rendering the UI within the main app loop.
use ratatui::{prelude::*, widgets::*};

use crate::user::User;

#[allow(unused_variables)]
pub fn render(
    frame: &mut Frame,
    input: &str,
    messages: &Vec<String>,
    logs: &Vec<String>,
    users: &Vec<User>,
) {
    let outer_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Min(1), Constraint::Length(3)])
        .split(frame.size());

    let inner_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(80), Constraint::Percentage(20)])
        .split(outer_layout[0]);

    // messages area
    let messages_area = inner_layout[0];
    let messages_widget = build_messages_widget(messages_area, messages);
    frame.render_widget(messages_widget, messages_area);

    // sidebar area
    let sidebar_area = inner_layout[1];

    // sidebar: users
    let usernames: Vec<String> = users.iter().map(|u| u.username.clone()).collect();
    let users_widget = build_users_widget(sidebar_area, &usernames);
    frame.render_widget(users_widget, sidebar_area);

    // sidebar: logs
    // let logs_widget = build_logs_widget(sidebar_area, logs);
    // frame.render_widget(logs_widget, sidebar_area);

    // input area
    let input_area = outer_layout[1];
    let input_block = Paragraph::new(input).block(Block::default().borders(Borders::ALL));
    frame.render_widget(input_block, input_area);

    // cursor
    let size = frame.size();
    let x = (input.len() + 1) as u16;
    let y = size.bottom() - 2;
    frame.set_cursor(x, y);
}

fn build_messages_widget(area: Rect, messages: &Vec<String>) -> List {
    let list_items = build_list_items(area, messages, 2, default_formatter);

    // pad items to push chat to the bottom
    let padded_items = pad_list(list_items.clone(), area.height - 2);

    let message_list = List::new(padded_items)
        .direction(ListDirection::TopToBottom)
        .block(Block::default().borders(Borders::ALL).title("Chat"));
    message_list
}

#[allow(dead_code)]
fn build_users_widget(area: Rect, usernames: &Vec<String>) -> List {
    let items = build_list_items(area, usernames, 2, default_formatter);
    let list = List::new(items)
        .direction(ListDirection::TopToBottom)
        .block(Block::default().borders(Borders::ALL).title("Users"));
    list
}

#[allow(dead_code)]
fn build_logs_widget(area: Rect, logs: &Vec<String>) -> List {
    let log_items = build_list_items(area, logs, 2, json_formatter);
    let log_list = List::new(log_items)
        .direction(ListDirection::TopToBottom)
        .block(Block::default().borders(Borders::ALL).title("Logs"));
    log_list
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
                items.push(ListItem::from(Text::from(sliced_lines)));
            }
            break;
        } else {
            items.push(ListItem::from(Text::from(lines.clone())));
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

    if let serde_json::Value::Array(elements) = json.clone() {
        if let Some(event) = elements.get(3).and_then(|e| e.as_str()) {
            if event != "presence_state" {
                return "".to_string();
            }
        }
    }

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
