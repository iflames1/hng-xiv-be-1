use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::models::{NewProfileData, ProfileFilters, ProfileRow};

pub async fn get_profile_by_id(pool: &PgPool, id: Uuid) -> Result<Option<ProfileRow>, sqlx::Error> {
    sqlx::query_as::<_, ProfileRow>(
        r#"
        SELECT id, name, gender, gender_probability, sample_size, age, age_group, country_id, country_probability, created_at
        FROM profiles
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn get_profile_by_name(
    pool: &PgPool,
    name: &str,
) -> Result<Option<ProfileRow>, sqlx::Error> {
    sqlx::query_as::<_, ProfileRow>(
        r#"
        SELECT id, name, gender, gender_probability, sample_size, age, age_group, country_id, country_probability, created_at
        FROM profiles
        WHERE name = $1
        "#,
    )
    .bind(name)
    .fetch_optional(pool)
    .await
}

pub async fn insert_profile(
    pool: &PgPool,
    profile: &NewProfileData,
) -> Result<ProfileRow, sqlx::Error> {
    let inserted = sqlx::query_as::<_, ProfileRow>(
        r#"
        INSERT INTO profiles (
            id,
            name,
            gender,
            gender_probability,
            sample_size,
            age,
            age_group,
            country_id,
            country_probability
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        ON CONFLICT (name) DO NOTHING
        RETURNING id, name, gender, gender_probability, sample_size, age, age_group, country_id, country_probability, created_at
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(&profile.name)
    .bind(&profile.gender)
    .bind(profile.gender_probability)
    .bind(profile.sample_size)
    .bind(profile.age)
    .bind(profile.age_group.as_str())
    .bind(&profile.country_id)
    .bind(profile.country_probability)
    .fetch_optional(pool)
    .await?;

    if let Some(row) = inserted {
        return Ok(row);
    }

    get_profile_by_name(pool, &profile.name)
        .await?
        .ok_or(sqlx::Error::RowNotFound)
}

pub async fn list_profiles(
    pool: &PgPool,
    filters: &ProfileFilters,
) -> Result<Vec<ProfileRow>, sqlx::Error> {
    let mut query = QueryBuilder::<Postgres>::new(
        r#"
        SELECT id, name, gender, gender_probability, sample_size, age, age_group, country_id, country_probability, created_at
        FROM profiles
        WHERE 1 = 1
        "#,
    );

    if let Some(gender) = &filters.gender {
        query.push(" AND gender = ").push_bind(gender);
    }

    if let Some(country_id) = &filters.country_id {
        query.push(" AND country_id = ").push_bind(country_id);
    }

    if let Some(age_group) = &filters.age_group {
        query.push(" AND age_group = ").push_bind(age_group);
    }

    query.push(" ORDER BY created_at DESC");

    query.build_query_as::<ProfileRow>().fetch_all(pool).await
}

pub async fn delete_profile(pool: &PgPool, id: Uuid) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM profiles WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected())
}
