use serde_json::Value;

pub fn build_batch(commands: Vec<Vec<String>>) -> Value {
    serde_json::json!(commands)
}
