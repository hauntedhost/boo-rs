use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::AppState;
use crate::app::Focus;
use crate::app::Onboarding;

pub fn build_widget(app: &AppState) -> (Paragraph, u16) {
    let input_width: u16;

    let input_paragraph = match app.onboarding {
        Onboarding::ConfirmingUsername => {
            // dim the input text if input is still the generated username
            let style = if app.input == app.user.username {
                Style::new().italic().dim()
            } else {
                Style::default()
            };

            let line = Line::from(vec![
                Span::raw("Enter a username"),
                Span::styled(" @ ", Style::default().italic()),
                Span::styled(app.input.clone(), style),
            ]);
            input_width = line.width() as u16;
            Paragraph::new(line)
        }
        Onboarding::ConfirmingRoom => {
            // dim the input if room is still the generated room name
            let style = if app.input == app.room {
                Style::new().italic().dim()
            } else {
                Style::default()
            };

            let line = Line::from(vec![
                Span::raw("Enter a room name"),
                Span::styled(" # ", Style::default().italic()),
                Span::styled(app.input.clone(), style),
            ]);
            input_width = line.width() as u16;
            Paragraph::new(line)
        }
        Onboarding::Completed => {
            if app.input.is_empty() {
                input_width = 0;
                let text = format!("Message #{}", app.room);
                let span = Span::raw(text).style(Style::new().italic().dim());
                Paragraph::new(span)
            } else {
                input_width = app.input.len() as u16;

                if app.input.starts_with("/") {
                    let command = app.input.trim()[1..].to_string();
                    let line = Line::from(vec![
                        Span::styled("/", Style::default().light_blue().bold().not_dim()),
                        Span::raw(command),
                    ]);
                    Paragraph::new(line)
                } else {
                    Paragraph::new(app.input.clone())
                }
            }
        }
    };

    let border_style = if app.ui_focus_area == Focus::Input {
        Style::new().not_dim()
    } else {
        Style::new().dim()
    };

    let input_block = input_paragraph.block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(border_style),
    );

    (input_block, input_width)
}
