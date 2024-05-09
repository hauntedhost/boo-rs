use crate::app::AppState;
use crate::app::Focus;
use crate::ui::styles::get_selection_style;
use crate::ui::styles::get_title_style;
use crate::ui::symbols::*;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render_widget(frame: &mut Frame, area: Rect, app: &mut AppState) {
    let rooms = app.get_rooms_with_counts();
    let widget = build_widget(&rooms, app.room.clone(), app.ui_focus_area);
    let selected_room = app.get_selected_or_current_room_index();
    app.ui_room_table_state.select(selected_room);
    frame.render_stateful_widget(widget, area, &mut app.ui_room_table_state);
}

fn build_widget(rooms: &Vec<(String, u32)>, current_room: String, focus: Focus) -> Table {
    let mut rows: Vec<Row> = vec![];
    for (room_name, user_count) in rooms {
        let room_name = format!("{room_name}");
        let style = get_selection_style(&room_name, &current_room);
        let row = Row::new(vec![format!("{room_name}"), format!("{user_count}")]).style(style);
        rows.push(row);
    }

    let border_style = if focus == Focus::Rooms {
        Style::new().not_dim()
    } else {
        Style::new().dim()
    };

    Table::new(rows, [Constraint::Fill(1), Constraint::Min(1)])
        .column_spacing(1)
        .flex(layout::Flex::Legacy)
        .highlight_symbol("> ")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(format!(" {HASH_SYMBOL} Rooms "))
                .title_style(get_title_style()),
        )
}
