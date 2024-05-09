use ratatui::prelude::*;
use ratatui::widgets::*;

use super::styles::get_selection_style;
use super::styles::get_title_style;

pub fn build_widget(app_user_uuid: String, users: &Vec<(String, String)>) -> Table {
    let mut rows: Vec<Row> = vec![];
    for (uuid, username) in users {
        let username = format!("{username}");
        let style = get_selection_style(&uuid, &app_user_uuid);
        let row = Row::new(vec![username]).style(style);
        rows.push(row);
    }

    Table::new(rows, [Constraint::Fill(1)])
        .column_spacing(1)
        .flex(layout::Flex::Legacy)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::new().dim())
                .title(" @ Users ")
                .title_style(get_title_style()),
        )
}
