use crate::app::AppState;
use crate::ui::styles::get_selection_style;
use crate::ui::styles::get_title_style;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render_widget(frame: &mut Frame, area: Rect, app: &AppState) {
    let users = app.get_uuid_username_pairs();
    let user_uuid = app.user.uuid.clone();
    let widget = build_widget(user_uuid, &users);
    frame.render_widget(widget, area);
}

fn build_widget(app_user_uuid: String, users: &Vec<(String, String)>) -> Table {
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
