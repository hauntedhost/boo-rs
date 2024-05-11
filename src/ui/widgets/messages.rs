use super::scrolbar;
use crate::app::AppState;
use crate::ui::format::Displayable;
use crate::ui::format::Format;
use crate::ui::math::area_height_minus_border;
use crate::ui::math::get_wrapped_line_counts;
use crate::ui::styles::get_title_style;
use crate::ui::symbols::*;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render_widget(frame: &mut Frame, area: Rect, app: &mut AppState) {
    let widget = build_widget(app, area);
    frame.render_widget(widget, area);

    scrolbar::render_scrollbar(
        frame,
        area,
        app.ui_messages_area_height,
        app.ui_messages_line_length,
        app.ui_messages_scrollbar_position,
    );
}

fn build_widget(app: &mut AppState, area: Rect) -> Paragraph {
    let messages = app.get_messages();
    let area_height = area_height_minus_border(area) as usize;

    let styled_messages: Vec<Line> = messages
        .iter()
        .map(|message| match message.format() {
            Format::SystemInternalMessage => Line::from(Span::styled(
                format!(
                    "{} {INTERNAL_MESSAGE_SYMBOL}",
                    message.display().to_string()
                ),
                Style::default().italic().dim(),
            )),
            Format::SystemPublicMessage => Line::from(Span::styled(
                format!("{}", message.display().to_string()),
                Style::default().light_blue().italic(),
            )),
            Format::UserMessage => {
                let text = message.display().to_string();
                let index = text.find(": ").expect("expected ':' in message");
                let username = text[..index + 1].to_string();
                let message = text[index + 1..].to_string();
                Line::from(vec![
                    Span::styled(username, Style::default().light_green()),
                    Span::raw(message),
                ])
            }
            _ => Line::from(Span::raw(message.display().to_string())),
        })
        .collect();

    let wrapped_line_counts = get_wrapped_line_counts(area, &messages);
    let wrapped_line_count: usize = wrapped_line_counts.iter().sum();

    app.set_messages_line_length_and_area_height(wrapped_line_count, area_height);
    let scrollbar_position = app.get_scrollbar_position();

    // TODO: conditional top padding
    // let padding = ?

    Paragraph::new(styled_messages)
        .wrap(Wrap { trim: false })
        .scroll((scrollbar_position as u16, 0))
        .block(
            Block::default()
                // .padding(padding)
                .borders(Borders::ALL)
                .border_style(Style::new().dim())
                .title(format!(" {CHAT_SYMBOL} Chat "))
                .title_style(get_title_style()),
        )
}
