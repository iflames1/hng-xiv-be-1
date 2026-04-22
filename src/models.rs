use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::str::FromStr;
use uuid::Uuid;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AgeGroup {
    Child,
    Teenager,
    Adult,
    Senior,
}

impl AgeGroup {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Child => "child",
            Self::Teenager => "teenager",
            Self::Adult => "adult",
            Self::Senior => "senior",
        }
    }
}

impl FromStr for AgeGroup {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "child" => Ok(Self::Child),
            "teenager" => Ok(Self::Teenager),
            "adult" => Ok(Self::Adult),
            "senior" => Ok(Self::Senior),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct ProfileRow {
    pub id: Uuid,
    pub name: String,
    pub gender: String,
    pub gender_probability: f64,
    pub age: i32,
    pub age_group: String,
    pub country_id: String,
    pub country_name: String,
    pub country_probability: f64,
    pub created_at: NaiveDateTime,
}

#[derive(Serialize)]
pub struct ProfileDetail {
    pub id: Uuid,
    pub name: String,
    pub gender: String,
    pub gender_probability: f64,
    pub age: i32,
    pub age_group: AgeGroup,
    pub country_id: String,
    pub country_name: String,
    pub country_probability: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct ProfileListItem {
    pub id: Uuid,
    pub name: String,
    pub gender: String,
    pub gender_probability: f64,
    pub age: i32,
    pub age_group: AgeGroup,
    pub country_id: String,
    pub country_name: String,
    pub country_probability: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<usize>,
    pub data: T,
}

#[derive(Debug, Clone)]
pub struct NewProfileData {
    pub name: String,
    pub gender: String,
    pub gender_probability: f64,
    pub age: i32,
    pub age_group: AgeGroup,
    pub country_id: String,
    pub country_name: String,
    pub country_probability: f64,
}

#[derive(Debug, Clone, Default)]
pub struct ProfileFilters {
    pub gender: Option<String>,
    pub age_group: Option<String>,
    pub country_id: Option<String>,
    pub min_age: Option<i32>,
    pub max_age: Option<i32>,
    pub min_gender_probability: Option<f64>,
    pub min_country_probability: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortBy {
    Age,
    CreatedAt,
    GenderProbability,
}

impl SortBy {
    pub fn from_query(value: Option<&str>) -> Result<Self, ()> {
        match value.map(|item| item.trim().to_ascii_lowercase()) {
            None => Ok(Self::CreatedAt),
            Some(value) if value == "age" => Ok(Self::Age),
            Some(value) if value == "created_at" => Ok(Self::CreatedAt),
            Some(value) if value == "gender_probability" => Ok(Self::GenderProbability),
            _ => Err(()),
        }
    }

    pub fn as_sql(&self) -> &'static str {
        match self {
            Self::Age => "age",
            Self::CreatedAt => "created_at",
            Self::GenderProbability => "gender_probability",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    Asc,
    Desc,
}

impl SortOrder {
    pub fn from_query(value: Option<&str>) -> Result<Self, ()> {
        match value.map(|item| item.trim().to_ascii_lowercase()) {
            None => Ok(Self::Desc),
            Some(value) if value == "asc" => Ok(Self::Asc),
            Some(value) if value == "desc" => Ok(Self::Desc),
            _ => Err(()),
        }
    }

    pub fn as_sql(&self) -> &'static str {
        match self {
            Self::Asc => "ASC",
            Self::Desc => "DESC",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Pagination {
    pub page: u32,
    pub limit: u32,
}

impl Pagination {
    pub fn offset(&self) -> i64 {
        i64::from(self.page.saturating_sub(1)) * i64::from(self.limit)
    }
}

#[derive(Debug, Clone)]
pub struct ListProfilesRequest {
    pub filters: ProfileFilters,
    pub sort_by: SortBy,
    pub order: SortOrder,
    pub pagination: Pagination,
}

#[derive(Debug, Clone)]
pub struct ParsedSearch {
    pub filters: ProfileFilters,
}

#[derive(Debug, Deserialize)]
pub struct SeedFile {
    pub profiles: Vec<SeedProfile>,
}

#[derive(Debug, Deserialize)]
pub struct SeedProfile {
    pub name: String,
    pub gender: String,
    pub gender_probability: f64,
    pub age: i32,
    pub age_group: String,
    pub country_id: String,
    pub country_name: String,
    pub country_probability: f64,
}

impl TryFrom<ProfileRow> for ProfileDetail {
    type Error = AppError;

    fn try_from(row: ProfileRow) -> Result<Self, Self::Error> {
        let age_group = AgeGroup::from_str(&row.age_group)
            .map_err(|_| AppError::internal("invalid age_group value in database"))?;

        Ok(Self {
            id: row.id,
            name: row.name,
            gender: row.gender,
            gender_probability: row.gender_probability,
            age: row.age,
            age_group,
            country_id: row.country_id,
            country_name: row.country_name,
            country_probability: row.country_probability,
            created_at: row.created_at.and_utc(),
        })
    }
}

impl TryFrom<ProfileRow> for ProfileListItem {
    type Error = AppError;

    fn try_from(row: ProfileRow) -> Result<Self, Self::Error> {
        let age_group = AgeGroup::from_str(&row.age_group)
            .map_err(|_| AppError::internal("invalid age_group value in database"))?;

        Ok(Self {
            id: row.id,
            name: row.name,
            gender: row.gender,
            gender_probability: row.gender_probability,
            age: row.age,
            age_group,
            country_id: row.country_id,
            country_name: row.country_name,
            country_probability: row.country_probability,
            created_at: row.created_at.and_utc(),
        })
    }
}
