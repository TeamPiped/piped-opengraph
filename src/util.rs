use std::env;

pub fn get_env_string(key: &str, default_value: String) -> String {
    match env::var(key) {
        Ok(var) => var,
        Err(_) => default_value,
    }
}
