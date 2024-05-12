use super::scrolbar;
use crate::app::message::Message as AppMessage;
use crate::app::AppState;
use crate::ui::math::area_height_minus_border;
use crate::ui::math::get_wrapped_line_counts;
use crate::ui::styles::get_title_style;
use crate::ui::symbols::*;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render_widget(frame: &mut Frame, area: Rect, app: &mut AppState) {
    let messages = app.get_messages();
    let area_height = area_height_minus_border(area) as usize;

    let lines: Vec<Line> = messages
        .iter()
        .map(|message| match message {
            AppMessage::SystemInternal(message) => Line::from(Span::styled(
                format!("{} {INTERNAL_MESSAGE_SYMBOL}", message),
                Style::default().italic().dim(),
            )),

            AppMessage::SystemPublic(message) => Line::from(Span::styled(
                format!("{}", message),
                Style::default().light_blue().italic(),
            )),
            AppMessage::User(message) => {
                let username = message.username.clone();
                let content = message.content.clone();

                Line::from(vec![
                    Span::styled(format!("{}: ", username), Style::default().light_green()),
                    Span::raw(content),
                ])
            }
        })
        .collect();

    let wrapped_line_counts = get_wrapped_line_counts(area, &messages);
    let wrapped_line_count: usize = wrapped_line_counts.iter().sum();

    app.set_messages_line_length_and_area_height(wrapped_line_count, area_height);
    let scrollbar_position = app.get_scrollbar_position();

    // TODO: conditional top padding
    // let padding = ?

    let widget = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((scrollbar_position as u16, 0))
        .block(
            Block::default()
                // .padding(padding)
                .borders(Borders::ALL)
                .border_style(Style::new().dim())
                .title(format!(" {CHAT_SYMBOL} Chat "))
                .title_style(get_title_style()),
        );

    frame.render_widget(widget, area);

    scrolbar::render_scrollbar(
        frame,
        area,
        app.ui_messages_area_height,
        app.ui_messages_line_length,
        app.ui_messages_scrollbar_position,
    );
}
