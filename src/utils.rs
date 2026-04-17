use axum::extract::rejection::JsonRejection;
use serde_json::Value;

use crate::{error::AppError, models::AgeGroup};

pub fn age_group_from_age(age: i32) -> AgeGroup {
    match age {
        0..=12 => AgeGroup::Child,
        13..=19 => AgeGroup::Teenager,
        20..=59 => AgeGroup::Adult,
        _ => AgeGroup::Senior,
    }
}

pub fn extract_name(payload: &Value) -> Result<String, AppError> {
    match payload.get("name") {
        Some(Value::String(name)) if !name.trim().is_empty() => Ok(name.trim().to_lowercase()),
        Some(Value::String(_)) | Some(Value::Null) => {
            Err(AppError::bad_request("Missing or empty name"))
        }
        Some(_) => Err(AppError::unprocessable_entity("Invalid type")),
        None => Err(AppError::bad_request("Missing or empty name")),
    }
}

pub fn normalize_optional_lower(value: Option<String>) -> Option<String> {
    value
        .map(|item| item.trim().to_lowercase())
        .filter(|item| !item.is_empty())
}

pub fn normalize_optional_upper(value: Option<String>) -> Option<String> {
    value
        .map(|item| item.trim().to_uppercase())
        .filter(|item| !item.is_empty())
}

pub fn map_json_rejection(rejection: JsonRejection) -> AppError {
    use axum::extract::rejection::JsonRejection::{
        BytesRejection, JsonDataError, JsonSyntaxError, MissingJsonContentType,
    };

    match rejection {
        JsonDataError(_) => AppError::unprocessable_entity("Invalid type"),
        JsonSyntaxError(_) | MissingJsonContentType(_) | BytesRejection(_) => {
            AppError::bad_request("Invalid request body")
        }
        _ => AppError::bad_request("Invalid request body"),
    }
}
