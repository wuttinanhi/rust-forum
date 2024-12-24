use serde_json::Value;

pub fn update_handlebars_data(data: &mut Value, key: &str, value: Value) {
    data.as_object_mut().unwrap().insert(key.to_string(), value);
}
