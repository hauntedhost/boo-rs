use serde_json::{json, to_string, Value};

const DEFAULT_TOPIC: &str = "relay:lobby";

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
    Raw(Value),
}

#[derive(Default, Debug)]
pub struct Message {
    pub event: Event,
    pub topic: Option<String>,
    pub payload: Payload,
    pub message_ref: usize,
    pub join_ref: Option<usize>,
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

    let topic = topic.unwrap_or(DEFAULT_TOPIC.to_string());
    let payload = parse_payload(payload);

    let json = to_string(&json!([join_ref, message_ref, topic, event, payload]))
        .expect("Failed to serialize JSON");

    json
}

fn parse_payload(payload: Payload) -> Value {
    match payload {
        Payload::Null => json!(null),
        Payload::Wrap(text) => json!({"message": text}),
        Payload::Raw(json) => json,
    }
}
