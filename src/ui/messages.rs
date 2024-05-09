// fn build_messages_widget(
//     area: Rect,
//     _room: String,
//     messages: &Vec<Message>,
//     scrollbar_position: usize,
// ) -> (Paragraph, usize, usize) {
//     // let wrapped_line_count = calculate_wrapped_line_count(area, messages);

//     let styled_messages: Vec<Line> = messages
//         .iter()
//         .map(|message| match message.format() {
//             // Format::Plaintext => Line::from(Span::raw(message.display().to_string())),
//             Format::SystemMessage => Line::from(Span::styled(
//                 message.display().to_string(),
//                 Style::default().italic().dim(),
//             )),
//             Format::UserMessage => {
//                 let text = message.display().to_string();
//                 let index = text.find(": ").expect("expected ':' in message");
//                 let username = text[..index + 1].to_string();
//                 let message = text[index + 1..].to_string();
//                 Line::from(vec![
//                     Span::styled(username, Style::default().light_green()),
//                     Span::raw(message),
//                 ])
//             }
//         })
//         .collect();

//     let wrapped_line_counts = get_wrapped_line_counts(area, messages);
//     let wrapped_line_count = wrapped_line_counts.iter().sum();
//     let wrapped_scrollbar_position: usize = wrapped_line_counts
//         .iter()
//         .take(scrollbar_position)
//         .sum::<usize>();

//     let paragraph = Paragraph::new(styled_messages)
//         .wrap(Wrap { trim: false })
//         .scroll((wrapped_scrollbar_position as u16, 0))
//         .block(
//             Block::default()
//                 // .padding(padding)
//                 .borders(Borders::ALL)
//                 .border_style(Style::new().dim())
//                 .title(format!(" {CHAT_SYMBOL} Chat "))
//                 .title_style(get_title_style()),
//         );

//     (paragraph, wrapped_scrollbar_position, wrapped_line_count)
// }
