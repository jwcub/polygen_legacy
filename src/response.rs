use rocket::serde::json::Value;

#[macro_export]
macro_rules! success {
    ($dat: expr) => {
        Ok(json!({"status": "success", "msg": $dat}))
    };
}

#[macro_export]
macro_rules! error {
    ($dat: expr) => {
        Err(json!({"status": "error", "msg": $dat}))
    };
}

pub type Response = Result<Value, Value>;
