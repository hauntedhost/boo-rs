mod client;
mod message;

use client::{Call, Client};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ezsockets::ClientConfig;
use rand::Rng;
use ratatui::{prelude::*, widgets::*};
use serde_json;
use std::env;
use std::io::{self, stdout};
use tokio::sync::mpsc::{self, Receiver};
use url::Url;

const DEFAULT_BASE_URL: &str = "ws://localhost:4000";

fn get_username() -> String {
    let mut rng = rand::thread_rng();
    let n: u32 = rng.gen_range(1..10_000);
    let username = env::var("NAME").unwrap_or(format!("guest{n}"));
    username
}

fn get_relay_url() -> Url {
    let mut base_url = env::var("RELAY_URL")
        .unwrap_or(DEFAULT_BASE_URL.to_string())
        .trim_end_matches('/')
        .to_string();
    base_url.push_str("/socket/websocket?vsn=2.0.0");
    Url::parse(&base_url).expect("Invalid relay URL")
}

#[derive(Default, Debug)]
pub struct Response {
    pub event: String,
    pub user: String,
    pub message: String,
}

fn extract_response(payload: &serde_json::Value) -> Option<Response> {
    let Some(event) = payload["event"].as_str() else {
        return None;
    };

    let Some(user) = payload["user"].as_str() else {
        return None;
    };

    let Some(message) = payload["message"].as_str() else {
        return None;
    };

    Some(Response {
        event: event.to_string(),
        user: user.to_string(),
        message: message.to_string(),
    })
}

fn parse_response(json: &String) -> Option<Response> {
    let Ok(value) = serde_json::from_str(json) else {
        return None;
    };

    if let serde_json::Value::Array(elements) = value {
        let Some(payload) = elements.get(4) else {
            return None;
        };

        if let Some(response) = payload.get("response") {
            return extract_response(response);
        }
    }
    None
}

fn get_last(messages: &Vec<String>, n: usize) -> Vec<String> {
    let start = if messages.len() > n {
        messages.len() - n
    } else {
        0
    };

    // Get the last 'take_last' messages or all messages if fewer than 'n'
    let mut latest_messages: Vec<String> = messages[start..].to_vec();

    // Pad with empty strings if there are less than 'n' messages
    while latest_messages.len() < n {
        latest_messages.insert(0, "".to_string());
    }

    latest_messages
}

fn ui(frame: &mut Frame, input: &str, messages: &Vec<String>, logs: &Vec<String>) {
    let outer_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Min(1), Constraint::Length(3)])
        .split(frame.size());

    let inner_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(outer_layout[0]);

    let messages_area = inner_layout[0];
    let logs_area = inner_layout[1];
    let input_area = outer_layout[1];

    // log area
    let last_logs = logs;
    let max_lines = (logs_area.height - 2) as usize;

    let mut line_count = 0;
    let mut log_list: Vec<ListItem> = vec![];
    for log in last_logs.iter().rev() {
        let json: serde_json::Value = serde_json::from_str(log).unwrap();
        let pretty_json = serde_json::to_string_pretty(&json).unwrap();
        let formatted_log = format!("=> {}", pretty_json.trim());

        let width = (logs_area.width - 2) as usize;
        let wrapped_texts = textwrap::wrap(&formatted_log, width);
        let lines = wrapped_texts
            .iter()
            .map(|s| Line::from(Span::raw(s.to_string())))
            .collect::<Vec<_>>();

        if line_count + lines.len() > max_lines {
            let lines_fit = max_lines - line_count;
            if lines_fit > 0 {
                // Only take as many lines as we can fit
                let mut sliced_lines = lines.into_iter().rev().take(lines_fit).collect::<Vec<_>>();
                sliced_lines.reverse();
                log_list.push(ListItem::from(Text::from(sliced_lines)));
            }
            break;
        } else {
            log_list.push(ListItem::from(Text::from(lines.clone())));
            line_count += lines.len();
        }
    }

    log_list.reverse();

    let logs = List::new(log_list)
        .direction(ListDirection::TopToBottom)
        .block(Block::default().borders(Borders::ALL).title("Logs"));

    frame.render_widget(logs, logs_area);

    // messages area
    let message_count = (messages_area.height - 2) as usize;
    let last_messages = get_last(messages, message_count);

    let mut list: Vec<ListItem> = vec![];
    for message in last_messages.iter() {
        let content = Line::from(Span::raw(message));
        list.push(ListItem::new(content));
    }

    let messages = List::new(list)
        .direction(ListDirection::TopToBottom)
        .block(Block::default().borders(Borders::ALL).title("Chat"));
    frame.render_widget(messages, messages_area);

    // input area
    let input_block = Paragraph::new(input).block(Block::default().borders(Borders::ALL));
    frame.render_widget(input_block, input_area);

    // cursor
    let size = frame.size();
    let x = (input.len() + 1) as u16;
    let y = size.bottom() - 2;
    frame.set_cursor(x, y);
}

fn handle_events(
    username: &mut String,
    input: &mut String,
    messages: &mut Vec<String>,
    logs: &mut Vec<String>,
    rx: &mut Receiver<String>,
    handle: &ezsockets::Client<client::Client>,
) -> io::Result<bool> {
    match rx.try_recv() {
        Ok(message_payload) => {
            logs.push(message_payload.clone());

            if let Some(response) = parse_response(&message_payload) {
                let Response {
                    event,
                    user,
                    message,
                } = response;

                if event.eq("joined") {
                    let message = format!("{user} has joined the chat!");
                    messages.push(message);
                } else if event.eq("shout") && !user.eq(username) {
                    let message = format!("{user}: {message}");
                    messages.push(message);
                }
            };
        }
        Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
            // No messages, do nothing
        }
        Err(_e) => {
            // Error, whatever
        }
    }

    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Esc
                || (key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c'))
            {
                return Ok(true);
            }

            if key.code == KeyCode::Enter {
                // TODO: fix username change logic
                //   1. push this message uniquely, e.g. "user x has changed their name to y"
                //   2. the server needs to handle the change too
                //   3. the rx.try_recv() also has to handle the name change broadcast
                // if input.starts_with("/username") {
                //     let prefix = "/username";
                //     let new_username = &input.trim()[prefix.len()..];
                //     if !new_username.is_empty() {
                //         *username = new_username.trim().to_string();
                //     } else {
                //         return Ok(false);
                //     }
                // }

                if input.len() > 0 {
                    let message = format!("{username}: {input}");
                    messages.push(message.clone());
                    handle
                        .call(Call::Shout(input.clone(), username.clone()))
                        .expect("call shout error");
                    input.clear();
                }
            } else if key.code == KeyCode::Backspace {
                input.pop();
            } else if let KeyCode::Char(c) = key.code {
                input.push(c);
            }
        }
    }

    Ok(false)
}

#[tokio::main]
async fn main() -> io::Result<()> {
    // tracing_subscriber::fmt().init();
    tracing_subscriber::fmt().with_env_filter("off").init();

    let relay_url = get_relay_url();
    let config = ClientConfig::new(relay_url);
    let (tx, mut rx) = mpsc::channel::<String>(32);
    let (handle, future) = ezsockets::connect(|handle| Client { handle, tx }, config).await;

    tokio::spawn(async move {
        future.await.unwrap();
    });

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut username = get_username();
    let mut input = "".to_string();
    let mut messages: Vec<String> = vec![];
    let mut logs: Vec<String> = vec![];
    let mut should_quit = false;

    handle
        .call(Call::Join(username.clone()))
        .expect("call join error");

    while !should_quit {
        terminal.draw(|f| ui(f, &input, &messages, &logs))?;
        should_quit = handle_events(
            &mut username,
            &mut input,
            &mut messages,
            &mut logs,
            &mut rx,
            &handle,
        )?;
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
