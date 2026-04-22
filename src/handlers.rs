use axum::{
    Json,
    extract::{Path, Query, State, rejection::JsonRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    db,
    error::AppError,
    external::ExternalApiClient,
    models::{ApiResponse, Pagination, ProfileDetail, ProfileListItem},
    state::AppState,
    utils::{extract_name, map_json_rejection, parse_list_profiles_query, parse_search_query},
};

#[derive(serde::Serialize)]
struct PagedProfilesResponse {
    status: &'static str,
    page: u32,
    limit: u32,
    total: i64,
    data: Vec<ProfileListItem>,
}

pub async fn create_profile(
    State(state): State<AppState>,
    body: Result<Json<Value>, JsonRejection>,
) -> Result<Response, AppError> {
    let Json(payload) = body.map_err(map_json_rejection)?;
    let name = extract_name(&payload)?;

    if let Some(existing) = db::get_profile_by_name(&state.pool, &name)
        .await
        .map_err(|_| AppError::internal("failed to read profile"))?
    {
        let response = ApiResponse {
            status: "success",
            message: Some("Profile already exists".to_string()),
            count: None,
            data: ProfileDetail::try_from(existing)?,
        };

        return Ok((StatusCode::OK, Json(response)).into_response());
    }

    let classified = ExternalApiClient::new(state.http_client.clone())
        .classify(&name)
        .await?;

    let inserted = db::insert_profile(&state.pool, &classified)
        .await
        .map_err(|_| AppError::internal("failed to save profile"))?;

    let response = ApiResponse {
        status: "success",
        message: None,
        count: None,
        data: ProfileDetail::try_from(inserted)?,
    };

    Ok((StatusCode::CREATED, Json(response)).into_response())
}

pub async fn get_profile(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response, AppError> {
    let profile_id =
        Uuid::parse_str(&id).map_err(|_| AppError::unprocessable_entity("Invalid type"))?;

    let profile = db::get_profile_by_id(&state.pool, profile_id)
        .await
        .map_err(|_| AppError::internal("failed to read profile"))?
        .ok_or_else(|| AppError::not_found("Profile not found"))?;

    let response = ApiResponse {
        status: "success",
        message: None,
        count: None,
        data: ProfileDetail::try_from(profile)?,
    };

    Ok((StatusCode::OK, Json(response)).into_response())
}

pub async fn list_profiles(
    State(state): State<AppState>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Response, AppError> {
    let request = parse_list_profiles_query(&query)?;

    let total = db::count_profiles(&state.pool, &request.filters)
        .await
        .map_err(|_| AppError::internal("failed to read profiles"))?;

    let profiles = db::list_profiles(&state.pool, &request)
        .await
        .map_err(|_| AppError::internal("failed to read profiles"))?;

    let data: Vec<ProfileListItem> = profiles
        .into_iter()
        .map(ProfileListItem::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    let response = PagedProfilesResponse {
        status: "success",
        page: request.pagination.page,
        limit: request.pagination.limit,
        total,
        data,
    };

    Ok((StatusCode::OK, Json(response)).into_response())
}

pub async fn search_profiles(
    State(state): State<AppState>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Response, AppError> {
    let (parsed, pagination) = parse_search_query(&query)?;

    let request = crate::models::ListProfilesRequest {
        filters: parsed.filters,
        sort_by: crate::models::SortBy::CreatedAt,
        order: crate::models::SortOrder::Desc,
        pagination: Pagination {
            page: pagination.page,
            limit: pagination.limit,
        },
    };

    let total = db::count_profiles(&state.pool, &request.filters)
        .await
        .map_err(|_| AppError::internal("failed to read profiles"))?;

    let profiles = db::list_profiles(&state.pool, &request)
        .await
        .map_err(|_| AppError::internal("failed to read profiles"))?;

    let data: Vec<ProfileListItem> = profiles
        .into_iter()
        .map(ProfileListItem::try_from)
        .collect::<Result<Vec<_>, _>>()?;

    let response = PagedProfilesResponse {
        status: "success",
        page: request.pagination.page,
        limit: request.pagination.limit,
        total,
        data,
    };

    Ok((StatusCode::OK, Json(response)).into_response())
}

pub async fn delete_profile(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response, AppError> {
    let profile_id =
        Uuid::parse_str(&id).map_err(|_| AppError::unprocessable_entity("Invalid type"))?;

    let deleted = db::delete_profile(&state.pool, profile_id)
        .await
        .map_err(|_| AppError::internal("failed to delete profile"))?;

    if deleted == 0 {
        return Err(AppError::not_found("Profile not found"));
    }

    Ok(StatusCode::NO_CONTENT.into_response())
}
