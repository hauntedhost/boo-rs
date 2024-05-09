pub mod format;
pub mod header;
pub mod help;
pub mod input;
pub mod logs;
pub mod math;
pub mod messages;
pub mod rooms;
pub mod styles;
pub mod symbols;
pub mod users;

use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::{AppState, RightSidebar};

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
    let messages_widget = messages::build_widget(app, messages_area);
    frame.render_widget(messages_widget, messages_area);

    // TODO: try to move this to messages.rs into build_scrollbar or something
    // -----------------------------------------------------------------------
    let messages_area_height = app.ui_messages_area_height;
    let messages_line_length = app.ui_messages_line_length;
    let scrollbar_position = app.ui_messages_scrollbar_position;

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"))
        .track_symbol(Some("│"));

    // conditionally show scrollbar
    // scrollbar is only visible if content_length > 0
    let content_length = if messages_line_length >= messages_area_height {
        messages_line_length
            .checked_sub(messages_area_height)
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
