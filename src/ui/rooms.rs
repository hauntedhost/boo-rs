use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::Focus;
use crate::ui::symbols::*;

use super::styles::get_selection_style;
use super::styles::get_title_style;

pub fn build_widget(rooms: &Vec<(String, u32)>, current_room: String, focus: Focus) -> Table {
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
