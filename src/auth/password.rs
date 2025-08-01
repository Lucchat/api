use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{http::StatusCode, Json};
use password_hash::SaltString;
use rand_core::OsRng;
use serde_json::{json, Value};

pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();

    Ok(hash)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(hash)?;
    let argon2 = Argon2::default();

    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

pub fn is_password_strong(password: &str) -> Result<bool, (StatusCode, Json<Value>)> {
    check_min_length(password, 12)?;
    check_lowercase(password)?;
    check_uppercase(password)?;
    check_digit(password)?;
    check_special_char(password)?;

    Ok(true)
}

fn check_min_length(password: &str, min_length: usize) -> Result<bool, (StatusCode, Json<Value>)> {
    if password.len() >= min_length {
        return Ok(true);
    }
    Err((
        StatusCode::BAD_REQUEST,
        Json(json!({ "error": {"code": 400, "message": "Password is too short" }})),
    ))
}

fn check_lowercase(password: &str) -> Result<bool, (StatusCode, Json<Value>)> {
    if password.chars().any(|c| c.is_lowercase()) {
        return Ok(true);
    }
    Err((
        StatusCode::BAD_REQUEST,
        Json(json!({ "error": {"code": 400, "message": "Password must contain at least one lowercase letter" }})),
    ))
}

fn check_uppercase(password: &str) -> Result<bool, (StatusCode, Json<Value>)> {
    if password.chars().any(|c| c.is_uppercase()) {
        return Ok(true);
    }
    Err((
        StatusCode::BAD_REQUEST,
        Json(json!({ "error": {"code": 400, "message": "Password must contain at least one uppercase letter" }})),
    ))
}

fn check_digit(password: &str) -> Result<bool, (StatusCode, Json<Value>)> {
    if password.chars().any(|c| c.is_digit(10)) {
        return Ok(true);
    }
    Err((
        StatusCode::BAD_REQUEST,
        Json(json!({ "error": {"code": 400, "message": "Password must contain at least one digit" }})),
    ))
}

fn check_special_char(password: &str) -> Result<bool, (StatusCode, Json<Value>)> {
    let regex = regex::Regex::new(r"\W").unwrap();
    if regex.is_match(password) {
        return Ok(true);
    }
    Err((
        StatusCode::BAD_REQUEST,
        Json(json!({ "error": {"code": 400, "message": "Password must contain at least one special character" }})),
    ))
}
