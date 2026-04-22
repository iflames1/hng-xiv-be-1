use axum::extract::rejection::JsonRejection;
use serde_json::Value;
use std::collections::{HashMap, HashSet};

use crate::{
    error::AppError,
    models::{
        AgeGroup, ListProfilesRequest, Pagination, ParsedSearch, ProfileFilters, SortBy, SortOrder,
    },
};

const DEFAULT_PAGE: u32 = 1;
const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 50;

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

pub fn parse_list_profiles_query(
    params: &HashMap<String, String>,
) -> Result<ListProfilesRequest, AppError> {
    let allowed: HashSet<&str> = [
        "gender",
        "age_group",
        "country_id",
        "min_age",
        "max_age",
        "min_gender_probability",
        "min_country_probability",
        "sort_by",
        "order",
        "page",
        "limit",
    ]
    .into_iter()
    .collect();

    if params.keys().any(|key| !allowed.contains(key.as_str())) {
        return Err(AppError::unprocessable_entity("Invalid query parameters"));
    }

    let gender = match params.get("gender") {
        Some(value) => Some(parse_gender(value)?),
        None => None,
    };

    let age_group = match params.get("age_group") {
        Some(value) => Some(parse_age_group(value)?),
        None => None,
    };

    let country_id = match params.get("country_id") {
        Some(value) => Some(parse_country_id(value)?),
        None => None,
    };

    let min_age = parse_optional_i32(params.get("min_age"))?;
    let max_age = parse_optional_i32(params.get("max_age"))?;

    if let (Some(min), Some(max)) = (min_age, max_age)
        && min > max
    {
        return Err(AppError::unprocessable_entity("Invalid query parameters"));
    }

    let min_gender_probability = parse_optional_f64(params.get("min_gender_probability"))?;
    let min_country_probability = parse_optional_f64(params.get("min_country_probability"))?;

    let sort_by = SortBy::from_query(params.get("sort_by").map(String::as_str))
        .map_err(|_| AppError::unprocessable_entity("Invalid query parameters"))?;

    let order = SortOrder::from_query(params.get("order").map(String::as_str))
        .map_err(|_| AppError::unprocessable_entity("Invalid query parameters"))?;

    let pagination = parse_pagination(params)?;

    Ok(ListProfilesRequest {
        filters: ProfileFilters {
            gender,
            age_group,
            country_id,
            min_age,
            max_age,
            min_gender_probability,
            min_country_probability,
        },
        sort_by,
        order,
        pagination,
    })
}

pub fn parse_search_query(
    params: &HashMap<String, String>,
) -> Result<(ParsedSearch, Pagination), AppError> {
    let allowed: HashSet<&str> = ["q", "page", "limit"].into_iter().collect();
    if params.keys().any(|key| !allowed.contains(key.as_str())) {
        return Err(AppError::unprocessable_entity("Invalid query parameters"));
    }

    let raw_query = params
        .get("q")
        .map(|value| value.trim())
        .ok_or_else(|| AppError::bad_request("Missing or empty parameter"))?;

    if raw_query.is_empty() {
        return Err(AppError::bad_request("Missing or empty parameter"));
    }

    let parsed = parse_natural_language(raw_query)?;
    let pagination = parse_pagination(params)?;

    Ok((parsed, pagination))
}

fn parse_pagination(params: &HashMap<String, String>) -> Result<Pagination, AppError> {
    let page = match params.get("page") {
        Some(value) => parse_u32(value)?,
        None => DEFAULT_PAGE,
    };

    let limit = match params.get("limit") {
        Some(value) => parse_u32(value)?,
        None => DEFAULT_LIMIT,
    };

    if page == 0 || limit == 0 || limit > MAX_LIMIT {
        return Err(AppError::unprocessable_entity("Invalid query parameters"));
    }

    Ok(Pagination { page, limit })
}

fn parse_u32(raw: &str) -> Result<u32, AppError> {
    let value = raw.trim();
    if value.is_empty() {
        return Err(AppError::unprocessable_entity("Invalid query parameters"));
    }

    value
        .parse::<u32>()
        .map_err(|_| AppError::unprocessable_entity("Invalid query parameters"))
}

fn parse_optional_i32(raw: Option<&String>) -> Result<Option<i32>, AppError> {
    raw.map(|value| {
        let value = value.trim();
        if value.is_empty() {
            return Err(AppError::unprocessable_entity("Invalid query parameters"));
        }

        value
            .parse::<i32>()
            .map_err(|_| AppError::unprocessable_entity("Invalid query parameters"))
    })
    .transpose()
}

fn parse_optional_f64(raw: Option<&String>) -> Result<Option<f64>, AppError> {
    raw.map(|value| {
        let value = value.trim();
        if value.is_empty() {
            return Err(AppError::unprocessable_entity("Invalid query parameters"));
        }

        let parsed = value
            .parse::<f64>()
            .map_err(|_| AppError::unprocessable_entity("Invalid query parameters"))?;

        if !(0.0..=1.0).contains(&parsed) {
            return Err(AppError::unprocessable_entity("Invalid query parameters"));
        }

        Ok(parsed)
    })
    .transpose()
}

fn parse_gender(value: &str) -> Result<String, AppError> {
    let normalized = value.trim().to_ascii_lowercase();
    if normalized == "male" || normalized == "female" {
        return Ok(normalized);
    }

    Err(AppError::unprocessable_entity("Invalid query parameters"))
}

fn parse_age_group(value: &str) -> Result<String, AppError> {
    let normalized = value.trim().to_ascii_lowercase();
    if matches!(
        normalized.as_str(),
        "child" | "teenager" | "adult" | "senior"
    ) {
        return Ok(normalized);
    }

    Err(AppError::unprocessable_entity("Invalid query parameters"))
}

fn parse_country_id(value: &str) -> Result<String, AppError> {
    let normalized = value.trim().to_ascii_uppercase();
    if normalized.len() == 2 && normalized.chars().all(|ch| ch.is_ascii_alphabetic()) {
        return Ok(normalized);
    }

    Err(AppError::unprocessable_entity("Invalid query parameters"))
}

fn parse_natural_language(query: &str) -> Result<ParsedSearch, AppError> {
    let normalized = normalize_search_text(query);
    let words: Vec<&str> = normalized.split_whitespace().collect();
    if words.is_empty() {
        return Err(AppError::bad_request("Unable to interpret query"));
    }

    let mut filters = ProfileFilters::default();

    let has_male = words
        .iter()
        .any(|word| matches!(*word, "male" | "males" | "man" | "men" | "boy" | "boys"));
    let has_female = words.iter().any(|word| {
        matches!(
            *word,
            "female" | "females" | "woman" | "women" | "girl" | "girls"
        )
    });

    if has_male && !has_female {
        filters.gender = Some("male".to_string());
    } else if has_female && !has_male {
        filters.gender = Some("female".to_string());
    }

    if words
        .iter()
        .any(|word| matches!(*word, "child" | "children" | "kid" | "kids"))
    {
        filters.age_group = Some("child".to_string());
    }

    if words
        .iter()
        .any(|word| matches!(*word, "teen" | "teens" | "teenager" | "teenagers"))
    {
        filters.age_group = Some("teenager".to_string());
    }

    if words.iter().any(|word| matches!(*word, "adult" | "adults")) {
        filters.age_group = Some("adult".to_string());
    }

    if words
        .iter()
        .any(|word| matches!(*word, "senior" | "seniors" | "elderly"))
    {
        filters.age_group = Some("senior".to_string());
    }

    if words.iter().any(|word| *word == "young") {
        filters.min_age = Some(filters.min_age.map_or(16, |current| current.max(16)));
        filters.max_age = Some(filters.max_age.map_or(24, |current| current.min(24)));
    }

    if let Some(min_age) = extract_age_threshold(&words, &["above", "over"]) {
        filters.min_age = Some(
            filters
                .min_age
                .map_or(min_age, |current| current.max(min_age)),
        );
    }

    if let Some(max_age) = extract_age_threshold(&words, &["below", "under"]) {
        filters.max_age = Some(
            filters
                .max_age
                .map_or(max_age, |current| current.min(max_age)),
        );
    }

    if let Some(country_id) = extract_country_filter(&words) {
        filters.country_id = Some(country_id);
    }

    if let (Some(min_age), Some(max_age)) = (filters.min_age, filters.max_age)
        && min_age > max_age
    {
        return Err(AppError::bad_request("Unable to interpret query"));
    }

    if filters.gender.is_none()
        && filters.age_group.is_none()
        && filters.country_id.is_none()
        && filters.min_age.is_none()
        && filters.max_age.is_none()
    {
        return Err(AppError::bad_request("Unable to interpret query"));
    }

    Ok(ParsedSearch { filters })
}

fn normalize_search_text(input: &str) -> String {
    input
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch.is_ascii_whitespace() {
                ch.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
}

fn extract_age_threshold(words: &[&str], tokens: &[&str]) -> Option<i32> {
    for (idx, word) in words.iter().enumerate() {
        if tokens.contains(word) {
            if let Some(next) = words.get(idx + 1)
                && let Ok(value) = next.parse::<i32>()
            {
                return Some(value);
            }
        }
    }

    None
}

fn extract_country_filter(words: &[&str]) -> Option<String> {
    for (idx, word) in words.iter().enumerate() {
        if *word != "from" {
            continue;
        }

        for span in [3, 2, 1] {
            let end = idx + 1 + span;
            if end > words.len() {
                continue;
            }

            let phrase = words[idx + 1..end].join(" ");
            if let Some(code) = country_keyword_to_iso(&phrase) {
                return Some(code.to_string());
            }
        }

        if let Some(next) = words.get(idx + 1)
            && next.len() == 2
            && next.chars().all(|ch| ch.is_ascii_alphabetic())
        {
            return Some(next.to_ascii_uppercase());
        }
    }

    None
}

fn country_keyword_to_iso(keyword: &str) -> Option<&'static str> {
    match keyword {
        "angola" => Some("AO"),
        "algeria" => Some("DZ"),
        "australia" => Some("AU"),
        "benin" => Some("BJ"),
        "botswana" => Some("BW"),
        "brazil" => Some("BR"),
        "burkina faso" => Some("BF"),
        "burundi" => Some("BI"),
        "cameroon" => Some("CM"),
        "canada" => Some("CA"),
        "cape verde" => Some("CV"),
        "central african republic" => Some("CF"),
        "china" => Some("CN"),
        "cote d ivoire" => Some("CI"),
        "democratic republic of the congo" => Some("CD"),
        "dr congo" => Some("CD"),
        "egypt" => Some("EG"),
        "eritrea" => Some("ER"),
        "ethiopia" => Some("ET"),
        "france" => Some("FR"),
        "gabon" => Some("GA"),
        "gambia" => Some("GM"),
        "ghana" => Some("GH"),
        "india" => Some("IN"),
        "kenya" => Some("KE"),
        "liberia" => Some("LR"),
        "madagascar" => Some("MG"),
        "malawi" => Some("MW"),
        "mali" => Some("ML"),
        "mauritania" => Some("MR"),
        "mauritius" => Some("MU"),
        "morocco" => Some("MA"),
        "mozambique" => Some("MZ"),
        "namibia" => Some("NA"),
        "nigeria" => Some("NG"),
        "republic of the congo" => Some("CG"),
        "rwanda" => Some("RW"),
        "senegal" => Some("SN"),
        "somalia" => Some("SO"),
        "south africa" => Some("ZA"),
        "sudan" => Some("SD"),
        "tanzania" => Some("TZ"),
        "tunisia" => Some("TN"),
        "uganda" => Some("UG"),
        "united kingdom" => Some("GB"),
        "united states" => Some("US"),
        "western sahara" => Some("EH"),
        "zambia" => Some("ZM"),
        "zimbabwe" => Some("ZW"),
        _ => None,
    }
}

pub fn country_name_from_iso(code: &str) -> Option<&'static str> {
    match code.trim().to_ascii_uppercase().as_str() {
        "AO" => Some("Angola"),
        "AU" => Some("Australia"),
        "BF" => Some("Burkina Faso"),
        "BI" => Some("Burundi"),
        "BJ" => Some("Benin"),
        "BR" => Some("Brazil"),
        "BW" => Some("Botswana"),
        "CA" => Some("Canada"),
        "CD" => Some("DR Congo"),
        "CF" => Some("Central African Republic"),
        "CG" => Some("Republic of the Congo"),
        "CI" => Some("Cote d'Ivoire"),
        "CM" => Some("Cameroon"),
        "CN" => Some("China"),
        "CV" => Some("Cape Verde"),
        "DZ" => Some("Algeria"),
        "EG" => Some("Egypt"),
        "EH" => Some("Western Sahara"),
        "ER" => Some("Eritrea"),
        "ET" => Some("Ethiopia"),
        "FR" => Some("France"),
        "GA" => Some("Gabon"),
        "GB" => Some("United Kingdom"),
        "GH" => Some("Ghana"),
        "GM" => Some("Gambia"),
        "IN" => Some("India"),
        "KE" => Some("Kenya"),
        "LR" => Some("Liberia"),
        "MA" => Some("Morocco"),
        "MG" => Some("Madagascar"),
        "ML" => Some("Mali"),
        "MR" => Some("Mauritania"),
        "MU" => Some("Mauritius"),
        "MW" => Some("Malawi"),
        "MZ" => Some("Mozambique"),
        "NA" => Some("Namibia"),
        "NG" => Some("Nigeria"),
        "RW" => Some("Rwanda"),
        "SD" => Some("Sudan"),
        "SN" => Some("Senegal"),
        "SO" => Some("Somalia"),
        "TN" => Some("Tunisia"),
        "TZ" => Some("Tanzania"),
        "UG" => Some("Uganda"),
        "US" => Some("United States"),
        "ZA" => Some("South Africa"),
        "ZM" => Some("Zambia"),
        "ZW" => Some("Zimbabwe"),
        _ => None,
    }
}
