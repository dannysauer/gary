use serde_yaml::Value;

pub fn merge(a: &mut Value, b: &Value) {
    match (a, b) {
        (Value::Mapping(a), Value::Mapping(b)) => {
            for (k, v) in b {
                merge(a.get_mut(k).unwrap(), v);
            }
        }
        (a, b) => {
            *a = b.clone();
        }
    }
}
