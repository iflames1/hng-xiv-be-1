use chrono::{DateTime, Utc};
use serde::Serialize;
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
    pub sample_size: i64,
    pub age: i32,
    pub age_group: String,
    pub country_id: String,
    pub country_probability: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct ProfileDetail {
    pub id: Uuid,
    pub name: String,
    pub gender: String,
    pub gender_probability: f64,
    pub sample_size: i64,
    pub age: i32,
    pub age_group: AgeGroup,
    pub country_id: String,
    pub country_probability: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct ProfileSummary {
    pub id: Uuid,
    pub name: String,
    pub gender: String,
    pub age: i32,
    pub age_group: AgeGroup,
    pub country_id: String,
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
    pub sample_size: i64,
    pub age: i32,
    pub age_group: AgeGroup,
    pub country_id: String,
    pub country_probability: f64,
}

#[derive(Debug, Clone, Default)]
pub struct ProfileFilters {
    pub gender: Option<String>,
    pub country_id: Option<String>,
    pub age_group: Option<String>,
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
            sample_size: row.sample_size,
            age: row.age,
            age_group,
            country_id: row.country_id,
            country_probability: row.country_probability,
            created_at: row.created_at,
        })
    }
}

impl TryFrom<ProfileRow> for ProfileSummary {
    type Error = AppError;

    fn try_from(row: ProfileRow) -> Result<Self, Self::Error> {
        let age_group = AgeGroup::from_str(&row.age_group)
            .map_err(|_| AppError::internal("invalid age_group value in database"))?;

        Ok(Self {
            id: row.id,
            name: row.name,
            gender: row.gender,
            age: row.age,
            age_group,
            country_id: row.country_id,
        })
    }
}
