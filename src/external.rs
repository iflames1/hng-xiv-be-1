use reqwest::Client;
use serde::Deserialize;

use crate::{
    error::AppError,
    models::NewProfileData,
    utils::{age_group_from_age, country_name_from_iso},
};

pub struct ExternalApiClient {
    client: Client,
}

#[derive(Debug, Deserialize)]
struct GenderizeResponse {
    gender: Option<String>,
    probability: Option<f64>,
    count: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct AgifyResponse {
    age: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct NationalizeCountry {
    country_id: Option<String>,
    probability: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct NationalizeResponse {
    country: Option<Vec<NationalizeCountry>>,
}

impl ExternalApiClient {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn classify(&self, name: &str) -> Result<NewProfileData, AppError> {
        let gender_url = format!("https://api.genderize.io?name={name}");
        let agify_url = format!("https://api.agify.io?name={name}");
        let nationalize_url = format!("https://api.nationalize.io?name={name}");

        let (genderize, agify, nationalize) = tokio::try_join!(
            self.fetch_genderize(&gender_url),
            self.fetch_agify(&agify_url),
            self.fetch_nationalize(&nationalize_url),
        )?;

        let gender = genderize
            .gender
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| AppError::upstream("Genderize"))?
            .to_lowercase();
        let gender_probability = genderize
            .probability
            .ok_or_else(|| AppError::upstream("Genderize"))?;
        if genderize.count.unwrap_or(0) == 0 {
            return Err(AppError::upstream("Genderize"));
        }

        let age = agify.age.ok_or_else(|| AppError::upstream("Agify"))?;
        let age_group = age_group_from_age(age);

        let country = nationalize
            .country
            .and_then(|countries| {
                countries
                    .into_iter()
                    .filter_map(|country| match (country.country_id, country.probability) {
                        (Some(country_id), Some(probability)) => Some((country_id, probability)),
                        _ => None,
                    })
                    .max_by(|left, right| {
                        left.1
                            .partial_cmp(&right.1)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
            })
            .ok_or_else(|| AppError::upstream("Nationalize"))?;

        let country_name = country_name_from_iso(&country.0)
            .ok_or_else(|| AppError::upstream("Nationalize"))?
            .to_string();

        Ok(NewProfileData {
            name: name.to_string(),
            gender,
            gender_probability,
            age,
            age_group,
            country_id: country.0,
            country_name,
            country_probability: country.1,
        })
    }

    async fn fetch_genderize(&self, url: &str) -> Result<GenderizeResponse, AppError> {
        self.fetch_json(url, "Genderize").await
    }

    async fn fetch_agify(&self, url: &str) -> Result<AgifyResponse, AppError> {
        self.fetch_json(url, "Agify").await
    }

    async fn fetch_nationalize(&self, url: &str) -> Result<NationalizeResponse, AppError> {
        self.fetch_json(url, "Nationalize").await
    }

    async fn fetch_json<T>(&self, url: &str, api_name: &str) -> Result<T, AppError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let response = self.client.get(url).send().await.map_err(|error| {
            tracing::warn!(api = api_name, url = url, error = %error, "upstream request failed");
            AppError::upstream(api_name)
        })?;

        if !response.status().is_success() {
            tracing::warn!(api = api_name, url = url, status = %response.status(), "upstream returned non-success status");
            return Err(AppError::upstream(api_name));
        }

        response
            .json::<T>()
            .await
            .map_err(|error| {
                tracing::warn!(api = api_name, url = url, error = %error, "failed to parse upstream response json");
                AppError::upstream(api_name)
            })
    }
}
