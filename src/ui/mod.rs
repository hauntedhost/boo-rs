pub mod format;
pub mod header;
pub mod help;
pub mod input;
pub mod logs;
pub mod rooms;
pub mod styles;
pub mod symbols;
pub mod users;

use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::{AppState, RightSidebar};
use crate::ui::symbols::*;
// use self::render_format::Format;

use self::format::{Displayable, Format};
use self::styles::get_title_style;

/// This module contains all code for rendering the UI within the main app loop.

#[allow(unused_variables)]
pub fn render(frame: &mut Frame, app: &mut AppState) {
    let AppState { user, input, .. } = app;

    if app.showing_help() {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(100)])
            .split(frame.size());
        let help_area = layout[0];
        let help_widget = help::build_widget(help_area);
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

    let (header_area, main_outer_layout) = (outer_layout[0], outer_layout[1]);

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

    // header area

    let header_widget = header::build_widget(
        app.input.clone(),
        app.get_username(),
        app.room.clone(),
        app.onboarding,
        app.socket_url.clone(),
        app.is_socket_active(),
    );
    frame.render_widget(header_widget, header_area);

    // users area
    let users = app.get_uuid_username_pairs();
    let users_widget = users::build_widget(app.user.uuid.clone(), &users);
    frame.render_widget(users_widget, left_sidebar_area);

    // messages area
    let message_area_height = area_height_minus_border(messages_area) as usize;
    let messages = app.get_messages();

    // TODO: move this back to build_users_widget
    // ------------------------------------------
    let styled_messages: Vec<Line> = messages
        .iter()
        .map(|message| match message.format() {
            Format::SystemMessage => Line::from(Span::styled(
                message.display().to_string(),
                Style::default().italic().dim(),
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

    // let plain_messages: Vec<Line> = messages
    //     .iter()
    //     .map(|message| Line::from(Span::raw(message.display().to_string())))
    //     .collect();

    let wrapped_line_counts = get_wrapped_line_counts(messages_area, &messages);
    let wrapped_line_count: usize = wrapped_line_counts.iter().sum();

    app.set_messages_line_length_and_area_height(wrapped_line_count, message_area_height);
    let scrollbar_position = app.get_scrollbar_position();

    let messages_widget = Paragraph::new(styled_messages)
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
    // ------------------------------------------
    frame.render_widget(messages_widget, messages_area);

    // TODO: try to move this to messages.rs into build_scrollbar or something
    // -----------------------------------------------------------------------
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"))
        .track_symbol(Some("│"));

    // conditionally show scrollbar
    // scrollbar is only visible if content_length > 0
    let content_length = if wrapped_line_count >= message_area_height {
        wrapped_line_count
            .checked_sub(message_area_height)
            .unwrap_or(0)
    } else {
        0
    };

    let mut scrollbar_state = ScrollbarState::default()
        .content_length(content_length)
        // .viewport_content_length(1)
        .position(scrollbar_position);
    // ------------------------------------------

    frame.render_stateful_widget(
        scrollbar,
        messages_area.inner(&Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut scrollbar_state,
    );

    // right_sidebar_area
    match app.ui_right_sidebar_view {
        RightSidebar::Rooms => {
            let rooms = app.get_rooms_with_counts();
            let rooms_widget = rooms::build_widget(&rooms, app.room.clone(), app.ui_focus_area);
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
            let logs_widget = logs::build_widget(right_sidebar_area, &logs);
            frame.render_widget(logs_widget, right_sidebar_area);
        }
    };

    // input area
    let (input_widget, input_width) = input::build_widget(&app);
    frame.render_widget(input_widget, input_area);

    // cursor
    let x = input_area.x + 1 + input_width;
    let y = input_area.y + 1;
    frame.set_cursor(x, y);
}

fn area_height_minus_border(area: Rect) -> u16 {
    area.height.checked_sub(2).unwrap_or(0)
}

fn area_width_minus_border(area: Rect) -> u16 {
    area.width.checked_sub(2).unwrap_or(0)
}

fn get_wrapped_line_counts<T>(area: Rect, items: &[T]) -> Vec<usize>
where
    T: Displayable,
{
    let width = area_width_minus_border(area) as usize;
    let mut line_counts = vec![];
    for item in items.iter() {
        let text = item.display().to_string();
        let wrapped_texts = textwrap::wrap(&text, width);
        line_counts.push(wrapped_texts.len());
    }
    line_counts
}
