use ratatui::{prelude::*, widgets::*};

pub fn render(frame: &mut Frame, input: &str, messages: &Vec<String>, logs: &Vec<String>) {
    let outer_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Min(1), Constraint::Length(3)])
        .split(frame.size());

    let inner_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(outer_layout[0]);

    let messages_area = inner_layout[0];
    let messages_widget = build_messages_widget(messages_area, messages);
    frame.render_widget(messages_widget, messages_area);

    // TODO: allow logs area to switch to username list!
    let logs_area = inner_layout[1];
    let logs_widget = build_logs_widget(logs_area, logs);
    frame.render_widget(logs_widget, logs_area);

    let input_area = outer_layout[1];
    let input_block = Paragraph::new(input).block(Block::default().borders(Borders::ALL));
    frame.render_widget(input_block, input_area);

    // cursor
    let size = frame.size();
    let x = (input.len() + 1) as u16;
    let y = size.bottom() - 2;
    frame.set_cursor(x, y);
}

// TODO: wrap lines like in logs_widgets
fn build_messages_widget(messages_area: Rect, messages: &Vec<String>) -> List {
    let message_count = (messages_area.height - 2) as usize;
    let last_messages = get_last(&messages, message_count);

    let mut items: Vec<ListItem> = vec![];
    for message in last_messages.iter() {
        let content = Line::from(Span::raw(message.to_string()));
        items.push(ListItem::new(content));
    }

    let message_list = List::new(items)
        .direction(ListDirection::TopToBottom)
        .block(Block::default().borders(Borders::ALL).title("Chat"));

    message_list
}

fn build_logs_widget(logs_area: Rect, logs: &Vec<String>) -> List {
    let last_logs = logs;
    let max_lines = (logs_area.height - 2) as usize;

    let mut line_count = 0;
    let mut log_items: Vec<ListItem> = vec![];
    for log in last_logs.iter().rev() {
        let json: serde_json::Value = serde_json::from_str(log).unwrap();
        let pretty_json = serde_json::to_string_pretty(&json).unwrap();
        let formatted_log = format!("=> {}", pretty_json.trim());

        let width = (logs_area.width - 2) as usize;
        let wrapped_texts = textwrap::wrap(&formatted_log, width);
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
                log_items.push(ListItem::from(Text::from(sliced_lines)));
            }
            break;
        } else {
            log_items.push(ListItem::from(Text::from(lines.clone())));
            line_count += lines.len();
        }
    }

    log_items.reverse();

    let log_list = List::new(log_items)
        .direction(ListDirection::TopToBottom)
        .block(Block::default().borders(Borders::ALL).title("Logs"));

    log_list
}

fn get_last(messages: &Vec<String>, n: usize) -> Vec<String> {
    let start = if messages.len() > n {
        messages.len() - n
    } else {
        0
    };

    // Get the last 'take_last' messages or all messages if fewer than 'n'
    let mut latest_messages: Vec<String> = messages[start..].to_vec();

    // Pad with empty strings if there are less than 'n' messages
    while latest_messages.len() < n {
        latest_messages.insert(0, "".to_string());
    }

    latest_messages
}
