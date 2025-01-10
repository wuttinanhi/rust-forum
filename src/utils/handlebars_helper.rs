use serde_json::Value;

pub fn update_handlebars_data(hb_data: &mut Value, key: &str, value: Value) {
    hb_data
        .as_object_mut()
        .unwrap()
        .insert(key.to_string(), value);
}
