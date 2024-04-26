use serde_json::{Result as SerdeResult, Value as SerdeValue};

// The server sends and receives messages as an array:
// [join_ref, message_ref, topic, event, payload]
pub type MessageArray = (Option<u32>, Option<u32>, String, String, SerdeValue);

pub fn parse_message_array(json_data: &str) -> SerdeResult<MessageArray> {
    let message_array: MessageArray = serde_json::from_str(json_data)?;
    Ok(message_array)
}

pub fn serialize_message_array(message_array: &MessageArray) -> SerdeResult<String> {
    let json = serde_json::to_string(message_array)?;
    Ok(json)
}
