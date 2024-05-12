use crate::app::AppState;
use crate::ui::styles::get_title_style;
use crate::ui::symbols::*;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render_widget(frame: &mut Frame, area: Rect, app: &AppState) {
    let logs = app.get_logs();

    let lines: Vec<Line> = logs
        .iter()
        .map(|log| {
            Line::from(vec![
                Span::styled(
                    log.logged_at.format("[%Y-%m-%d %H:%M:%S] ").to_string(),
                    Style::default().dim(),
                ),
                Span::raw(format!("{:?}", log.response)),
            ])
        })
        .collect();

    let widget = Paragraph::new(lines).wrap(Wrap { trim: false }).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::new().dim())
            .title(format!(" {LOG_SYMBOL} Logs "))
            .title_style(get_title_style()),
    );

    frame.render_widget(widget, area);
}
