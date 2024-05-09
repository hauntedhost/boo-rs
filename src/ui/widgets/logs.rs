use crate::app::AppState;
use crate::ui::format::Displayable;
use crate::ui::format::Format;
use crate::ui::styles::get_title_style;
use crate::ui::symbols::*;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render_widget(frame: &mut Frame, area: Rect, app: &AppState) {
    let logs = app.get_logs();
    let widget = build_widget(area, &logs);
    frame.render_widget(widget, area);
}

fn build_widget(area: Rect, logs: &Vec<String>) -> List {
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

// Example: filter presence_state events
// if let serde_json::Value::Array(elements) = json.clone() {
//     if let Some(event) = elements.get(3).and_then(|e| e.as_str()) {
//         if event != "presence_state" {
//             return "".to_string();
//         }
//     }
// }
fn json_formatter(string: &String) -> String {
    let json: serde_json::Value = serde_json::from_str(string).unwrap();
    let pretty_json = serde_json::to_string_pretty(&json).unwrap();
    let formatted_log = format!("> {}", pretty_json.trim());

    formatted_log
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
            Format::SystemMessage => Style::default().italic().dim(),
            _ => Style::default(),
        };
        let formatted_item = formatter(&item_content);

        // Wrap text to fit the area width
        let width = (area.width - padding) as usize;
        let wrapped_texts = textwrap::wrap(&formatted_item, width);

        let lines = wrapped_texts
            .iter()
            .map(|s| Line::from(Span::raw(s.to_string())))
            .collect::<Vec<_>>();

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
