pub mod math;
pub mod styles;
pub mod symbols;
pub mod widgets;
use self::widgets::{header, help, input, messages, sidebars, users};
use crate::app::{AppState, RightSidebar};
use ratatui::prelude::*;

/// This module contains code for rendering the UI within the main app loop.

#[allow(unused_variables)]
pub fn render(frame: &mut Frame, app: &mut AppState) {
    let AppState { user, input, .. } = app;

    // Layouts
    // TODO: move to ui/layouts.rs

    // outer vertical layout
    // +-----------------------------------------------+
    // | header_area                                   |
    // +-----------------------------------------------+
    // +-----------------------------------------------+
    // | main_outer_layout                             |
    // |                                               |
    // |                                               |
    // |                                               |
    // +-----------------------------------------------+

    let outer_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Length(3), Constraint::Min(1)])
        .split(frame.size());

    let (header_area, main_outer_layout) = (outer_layout[0], outer_layout[1]);

    // horizontal layout inside main_outer_layout
    // +---------+ +-----------------------+ +---------+
    // | left    | | messages_outer_layout | | right   |
    // | sidebar | |                       | | sidebar |
    // | area    | |                       | | area    |
    // |         | |                       | |         |
    // |         | |                       | |         |
    // |         | |                       | |         |
    // |         | |                       | |         |
    // |         | |                       | |         |
    // +---------+ +-----------------------+ +---------+

    let (left_sidebar_width, messages_width, right_sidebar_width) = match app.ui_right_sidebar_view
    {
        RightSidebar::Rooms => (22, 56, 22),
        RightSidebar::Logs => (15, 30, 55),
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
    // +-----------------------------------------------+
    // | messages_area                                 |
    // |                                               |
    // |                                               |
    // |                                               |
    // +-----------------------------------------------+
    // +-----------------------------------------------+
    // | input_area                                    |
    // +-----------------------------------------------+

    let messages_layout = Layout::default()
        .direction(Direction::Vertical)
        .spacing(0)
        .margin(0)
        .constraints(vec![Constraint::Min(1), Constraint::Length(3)])
        .split(messages_outer_layout);

    let (messages_area, input_area) = (messages_layout[0], messages_layout[1]);

    // Widgets

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

    header::render_widget(frame, header_area, app);
    users::render_widget(frame, left_sidebar_area, app);
    messages::render_widget(frame, messages_area, app);
    sidebars::render_right_sidebar_widget(frame, right_sidebar_area, app);
    input::render_widget(frame, input_area, app);

    // Cursor
    // Clamp x poition to input area width (see input::render_widget for horizontal scroll logic)
    let input_area_width = input_area.width - 2;
    let x = if app.ui_input_width >= input_area_width {
        input_area.x + input_area_width
    } else {
        input_area.x + 1 + app.ui_input_width
    };

    let y = input_area.y + 1;
    frame.set_cursor(x, y);
}
