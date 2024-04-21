#[derive(Default, Debug)]
pub enum Event {
    Join,
    #[default]
    Shout,
}

#[allow(dead_code)]
#[derive(Default, Debug)]
pub enum Payload {
    #[default]
    Null,
    Wrap(String),
    Raw(String),
}

#[derive(Default, Debug)]
pub struct Message {
    pub event: Event,
    pub topic: Option<String>,
    pub payload: Payload,
    pub message_ref: Option<String>,
    pub join_ref: Option<String>,
}

pub fn message_text(message: Message) -> String {
    let Message {
        event,
        topic,
        payload,
        message_ref,
        join_ref,
    } = message;

    let event = match event {
        Event::Join => "phx_join",
        Event::Shout => "shout",
    };

    let topic = topic.unwrap_or("relay:lobby".to_string());

    let payload = match payload {
        Payload::Null => "null".to_string(),
        Payload::Wrap(text) => format!(r#"{{"message": "{text}"}}"#),
        Payload::Raw(json) => json,
    };

    let message_ref = quoted_or_null_string(message_ref);
    let join_ref = quoted_or_null_string(join_ref);

    format!(r#"[{join_ref}, {message_ref}, "{topic}", "{event}", {payload}]"#)
}

fn quoted_or_null_string(s: Option<String>) -> String {
    match s {
        Some(string) => format!("\"{string}\""),
        None => "null".to_string(),
    }
}
