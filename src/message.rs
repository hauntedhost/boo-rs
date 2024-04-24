use serde_json::{from_str, json, to_string};

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
    let payload = parse_payload(payload);
    let message_ref = json_stringify(message_ref);
    let join_ref = json_stringify(join_ref);

    format!(r#"[{join_ref}, {message_ref}, "{topic}", "{event}", {payload}]"#)
}

fn parse_payload(payload: Payload) -> String {
    let json = match payload {
        Payload::Null => json!(null),
        Payload::Wrap(text) => json!({"message": text}),
        Payload::Raw(json_str) => from_str(&json_str).expect("Failed to parse JSON string"),
    };

    to_string(&json).expect("Failed to serialize JSON")
}

fn json_stringify(s: Option<String>) -> String {
    match s {
        Some(string) => to_string(&string).unwrap(),
        None => to_string(&json!(null)).unwrap(),
    }
}
