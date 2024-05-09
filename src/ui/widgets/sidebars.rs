use super::{logs, rooms};
use crate::app::{AppState, RightSidebar};
use ratatui::prelude::*;

pub fn render_right_sidebar_widget(frame: &mut Frame, area: Rect, app: &mut AppState) {
    match app.ui_right_sidebar_view {
        RightSidebar::Rooms => rooms::render_widget(frame, area, app),
        RightSidebar::Logs => logs::render_widget(frame, area, app),
    };
}
