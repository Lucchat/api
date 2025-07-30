use axum::{http::StatusCode, Json};
use serde_json::{json, Value};

pub fn error_response(status: StatusCode, message: Option<&str>) -> (StatusCode, Json<Value>) {
    let default_message = match status {
        StatusCode::BAD_REQUEST => "Bad request",
        StatusCode::UNAUTHORIZED => "Unauthorized",
        StatusCode::FORBIDDEN => "Forbidden",
        StatusCode::NOT_FOUND => "Resource not found",
        StatusCode::CONFLICT => "Conflict",
        StatusCode::INTERNAL_SERVER_ERROR => "Internal server error",
        _ => "An error occurred",
    };

    let msg = message.unwrap_or(default_message);

    (
        status,
        Json(json!({
            "error": {
                "code": status.as_u16(),
                "message": msg,
            }
        })),
    )
}
