use crate::app::AppState;
use crate::ui::format::Displayable;
use crate::ui::styles::get_title_style;
use crate::ui::symbols::*;
use ratatui::prelude::*;
use ratatui::widgets::*;

#[derive(Clone, Default, Debug)]
pub struct Log {
    pub json_payload: String,
}

pub fn render_widget(frame: &mut Frame, area: Rect, app: &AppState) {
    let logs = app.get_logs();
    let widget = build_widget(area, &logs);
    frame.render_widget(widget, area);
}

fn build_widget(area: Rect, logs: &Vec<Log>) -> List {
    let (log_items, _line_count) = build_list_items(area, logs, 2);
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

fn build_list_items<T>(area: Rect, all_items: &[T], padding: u16) -> (Vec<ListItem>, usize)
where
    T: Displayable,
{
    let max_lines = (area.height - padding) as usize;
    let mut line_count = 0;
    let mut items: Vec<ListItem> = vec![];
    for item in all_items.iter().rev() {
        // Wrap text to fit the area width
        let formatted_item = item.display();
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

    (items, line_count)
}
