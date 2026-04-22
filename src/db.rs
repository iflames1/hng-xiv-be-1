use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::models::{
    ListProfilesRequest, NewProfileData, ProfileFilters, ProfileRow, SeedProfile, SortBy,
};

pub async fn get_profile_by_id(pool: &PgPool, id: Uuid) -> Result<Option<ProfileRow>, sqlx::Error> {
    sqlx::query_as::<_, ProfileRow>(
        r#"
        SELECT id, name, gender, gender_probability, age, age_group, country_id, country_name, country_probability, created_at
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
        SELECT id, name, gender, gender_probability, age, age_group, country_id, country_name, country_probability, created_at
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
            name,
            gender,
            gender_probability,
            age,
            age_group,
            country_id,
            country_name,
            country_probability
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (name) DO NOTHING
        RETURNING id, name, gender, gender_probability, age, age_group, country_id, country_name, country_probability, created_at
        "#,
    )
    .bind(&profile.name)
    .bind(&profile.gender)
    .bind(profile.gender_probability)
    .bind(profile.age)
    .bind(profile.age_group.as_str())
    .bind(&profile.country_id)
    .bind(&profile.country_name)
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
    request: &ListProfilesRequest,
) -> Result<Vec<ProfileRow>, sqlx::Error> {
    let mut query = build_profiles_base_query(&request.filters);

    query
        .push(" ORDER BY ")
        .push(request.sort_by.as_sql())
        .push(" ")
        .push(request.order.as_sql());

    if !matches!(request.sort_by, SortBy::CreatedAt) {
        query.push(", created_at DESC");
    }

    query
        .push(" LIMIT ")
        .push_bind(i64::from(request.pagination.limit))
        .push(" OFFSET ")
        .push_bind(request.pagination.offset());

    query.build_query_as::<ProfileRow>().fetch_all(pool).await
}

pub async fn count_profiles(pool: &PgPool, filters: &ProfileFilters) -> Result<i64, sqlx::Error> {
    let mut query =
        QueryBuilder::<Postgres>::new("SELECT COUNT(*) as count FROM profiles WHERE 1=1");
    apply_profile_filters(&mut query, filters);

    query.build_query_scalar::<i64>().fetch_one(pool).await
}

fn build_profiles_base_query(filters: &ProfileFilters) -> QueryBuilder<'static, Postgres> {
    let mut query = QueryBuilder::<Postgres>::new(
        r#"
        SELECT id, name, gender, gender_probability, age, age_group, country_id, country_name, country_probability, created_at
        FROM profiles
        WHERE 1=1
        "#,
    );

    apply_profile_filters(&mut query, filters);
    query
}

fn apply_profile_filters<'args>(
    query: &mut QueryBuilder<'args, Postgres>,
    filters: &ProfileFilters,
) {
    if let Some(gender) = &filters.gender {
        query.push(" AND gender = ").push_bind(gender.clone());
    }

    if let Some(country_id) = &filters.country_id {
        query
            .push(" AND country_id = ")
            .push_bind(country_id.clone());
    }

    if let Some(age_group) = &filters.age_group {
        query.push(" AND age_group = ").push_bind(age_group.clone());
    }

    if let Some(min_age) = filters.min_age {
        query.push(" AND age >= ").push_bind(min_age);
    }

    if let Some(max_age) = filters.max_age {
        query.push(" AND age <= ").push_bind(max_age);
    }

    if let Some(min_gender_probability) = filters.min_gender_probability {
        query
            .push(" AND gender_probability >= ")
            .push_bind(min_gender_probability);
    }

    if let Some(min_country_probability) = filters.min_country_probability {
        query
            .push(" AND country_probability >= ")
            .push_bind(min_country_probability);
    }
}

pub async fn seed_profiles(pool: &PgPool, profiles: &[SeedProfile]) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    for profile in profiles {
        sqlx::query(
            r#"
            INSERT INTO profiles (
                name,
                gender,
                gender_probability,
                age,
                age_group,
                country_id,
                country_name,
                country_probability
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (name) DO NOTHING
            "#,
        )
        .bind(&profile.name)
        .bind(profile.gender.trim().to_ascii_lowercase())
        .bind(profile.gender_probability)
        .bind(profile.age)
        .bind(profile.age_group.trim().to_ascii_lowercase())
        .bind(profile.country_id.trim().to_ascii_uppercase())
        .bind(profile.country_name.trim())
        .bind(profile.country_probability)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await
}

pub async fn delete_profile(pool: &PgPool, id: Uuid) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM profiles WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected())
}
