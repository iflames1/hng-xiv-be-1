# Name Classification Profiles API

Rust + Axum + SQLx service for creating and querying profiles.

## Endpoints

- `POST /api/profiles`
- `GET /api/profiles/{id}`
- `DELETE /api/profiles/{id}`
- `GET /api/profiles` (advanced filtering, sorting, pagination)
- `GET /api/profiles/search` (rule-based natural language search + pagination)

## Profiles Table (Exact Structure)

| Field               | Type       | Notes                                     |
| ------------------- | ---------- | ----------------------------------------- |
| id                  | UUID v7    | Primary key, default `uuid_generate_v7()` |
| name                | VARCHAR    | Unique, full name                         |
| gender              | VARCHAR    | `male` or `female`                        |
| gender_probability  | FLOAT      | Confidence score                          |
| age                 | INT        | Exact age                                 |
| age_group           | VARCHAR    | `child`, `teenager`, `adult`, `senior`    |
| country_id          | VARCHAR(2) | ISO code (`NG`, `BJ`, etc.)               |
| country_name        | VARCHAR    | Full country name                         |
| country_probability | FLOAT      | Confidence score                          |
| created_at          | TIMESTAMP  | Auto-generated                            |

## Seeding

- Seed source: [assets/seed_profiles.json](assets/seed_profiles.json)
- Seeding runs on server start (after migrations)
- Seeding inserts omit `id`; DB default generates UUID automatically
- Seed insert uses `ON CONFLICT (name) DO NOTHING` to avoid duplicates

## GET /api/profiles

Supports all filters together, plus sorting and pagination.

### Query Parameters

Filters:

- `gender`
- `age_group`
- `country_id`
- `min_age`
- `max_age`
- `min_gender_probability`
- `min_country_probability`

Sorting:

- `sort_by`: `age` | `created_at` | `gender_probability`
- `order`: `asc` | `desc`

Pagination:

- `page`: default `1`
- `limit`: default `10`, max `50`

Example:

```http
GET /api/profiles?gender=male&country_id=NG&min_age=25&sort_by=age&order=desc&page=1&limit=10
```

Success (`200`):

```json
{
	"status": "success",
	"page": 1,
	"limit": 10,
	"total": 2026,
	"data": [
		{
			"id": "b3f9c1e2-7d4a-4c91-9c2a-1f0a8e5b6d12",
			"name": "emmanuel",
			"gender": "male",
			"gender_probability": 0.99,
			"age": 34,
			"age_group": "adult",
			"country_id": "NG",
			"country_name": "Nigeria",
			"country_probability": 0.85,
			"created_at": "2026-04-01T12:00:00Z"
		}
	]
}
```

## GET /api/profiles/search

```http
GET /api/profiles/search?q=young males from nigeria&page=1&limit=10
```

Rule-based parsing only (no AI/LLMs). Pagination works the same as `/api/profiles`.

Success response shape is identical to `GET /api/profiles`.

When query cannot be interpreted:

```json
{ "status": "error", "message": "Unable to interpret query" }
```

## Natural Language Parser Approach

The parser is deterministic and keyword-based:

1. Normalize input:

- Lowercase text
- Remove punctuation
- Tokenize by whitespace

2. Extract supported intents:

- Gender keywords
- Age-group keywords
- Age threshold keywords (`above`, `over`, `below`, `under`)
- Country phrase after `from ...`
- Special keyword `young`

3. Build filters:

- Merge all detected conditions into one `ProfileFilters` object
- All resulting filters are combined with logical AND in SQL

4. Validate interpretation:

- If no supported condition is found, return `Unable to interpret query`

### Supported Keyword Mapping

- `young` -> `min_age=16` and `max_age=24`
- `male`, `males`, `man`, `men`, `boy`, `boys` -> `gender=male`
- `female`, `females`, `woman`, `women`, `girl`, `girls` -> `gender=female`
- `child`, `children`, `kid`, `kids` -> `age_group=child`
- `teen`, `teens`, `teenager`, `teenagers` -> `age_group=teenager`
- `adult`, `adults` -> `age_group=adult`
- `senior`, `seniors`, `elderly` -> `age_group=senior`
- `above N` or `over N` -> `min_age=N`
- `below N` or `under N` -> `max_age=N`
- `from <country>` -> mapped to `country_id` (supported country keyword map)

Examples:

- `young males` -> `gender=male` + `min_age=16` + `max_age=24`
- `females above 30` -> `gender=female` + `min_age=30`
- `people from angola` -> `country_id=AO`
- `adult males from kenya` -> `gender=male` + `age_group=adult` + `country_id=KE`
- `male and female teenagers above 17` -> `age_group=teenager` + `min_age=17`

## Parser Limitations

- Only keyword/rule matching is supported; no semantic or contextual inference.
- Country extraction expects `from ...` pattern.
- Country recognition is limited to the built-in mapping list.
- Comparative expressions beyond supported tokens are not handled (for example: `at least`, `not older than`, `between`).
- Complex negation and exclusion queries are not supported.
- Multiple unrelated clauses with conflicting meanings may be reduced to best-effort filters.

## Error Contract

All errors:

```json
{ "status": "error", "message": "<error message>" }
```

Status rules:

- `400 Bad Request` -> missing or empty parameter
- `422 Unprocessable Entity` -> invalid parameter type/value
- `404 Not Found` -> profile not found
- `500` / `502` -> server/upstream failures

Invalid query parameters return:

```json
{ "status": "error", "message": "Invalid query parameters" }
```

## Performance Notes

To reduce expensive scans under filtering/sorting:

- Dedicated indexes exist for common filter/sort fields
- Composite index exists for common multi-filter path (`gender`, `country_id`, `age`)
- Queries are paginated with `LIMIT/OFFSET`
- Count and page data queries share the same filters

## Run

```bash
cargo fmt --all
cargo check
cargo run
```
