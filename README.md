# Name Classification Profiles API

A Rust backend service that:

- Accepts a name
- Calls three external APIs (Genderize, Agify, Nationalize)
- Applies classification rules
- Stores the resulting profile in PostgreSQL
- Exposes endpoints to create, list, fetch, and delete profiles

Built with Axum, Tokio, SQLx, and Reqwest.

## Features

- `POST /api/profiles` creates a profile from a name
- De-duplicates by name (returns existing profile instead of creating a new one)
- `GET /api/profiles/{id}` fetches one profile by UUID
- `GET /api/profiles` lists profiles with optional filters
- `DELETE /api/profiles/{id}` deletes a profile
- Uses `PORT` from environment with fallback to `3000`
- Graceful shutdown using Tokio signal handling (`Ctrl+C`, and `SIGTERM` on Unix)

## External APIs Used

- Genderize: `https://api.genderize.io?name={name}`
- Agify: `https://api.agify.io?name={name}`
- Nationalize: `https://api.nationalize.io?name={name}`

No API keys required.

## Classification Rules

- Age group from Agify:
- `0-12` -> `child`
- `13-19` -> `teenager`
- `20-59` -> `adult`
- `60+` -> `senior`
- Nationality: highest probability country from Nationalize response

## Tech Stack

- Rust (edition 2024)
- Axum
- Tokio
- SQLx + PostgreSQL
- Reqwest
- Serde
- Tracing

## Project Structure

```text
src/
	app.rs        # app startup, DB pool, routes, graceful shutdown
	main.rs       # entrypoint
	handlers.rs   # HTTP handlers
	external.rs   # external API client + classification orchestration
	db.rs         # DB queries
	models.rs     # API/DB models and age-group enum
	utils.rs      # request parsing and normalization helpers
	error.rs      # unified API error responses
	state.rs      # shared app state (DB pool + HTTP client)
migrations/
	*.sql         # schema migrations
```

## Prerequisites

- Rust toolchain installed
- PostgreSQL database (Neon or local)

## Environment Variables

Create a `.env` file:

```env
DATABASE_URL="postgresql://<user>:<password>@<host>/<db>?sslmode=require"
PORT=8080
```

- `DATABASE_URL` is required
- `PORT` is optional (defaults to `3000`)

## Run Locally

1. Install dependencies and build:

```bash
cargo build
```

2. Run the app:

```bash
cargo run
```

3. The server listens on:

- `0.0.0.0:${PORT}` if `PORT` is set
- `0.0.0.0:3000` otherwise

## Database Migrations

Migrations are run automatically on startup.

If startup fails, verify:

- `DATABASE_URL` is correct
- database user has permissions to create table/index

## API

Base URL (local): `http://localhost:3000` or your configured port.

### 1) Create Profile

`POST /api/profiles`

Request:

```json
{
	"name": "ella"
}
```

Success (created): `201`

```json
{
	"status": "success",
	"data": {
		"id": "b3f9c1e2-7d4a-4c91-9c2a-1f0a8e5b6d12",
		"name": "ella",
		"gender": "female",
		"gender_probability": 0.99,
		"sample_size": 1234,
		"age": 46,
		"age_group": "adult",
		"country_id": "DRC",
		"country_probability": 0.85,
		"created_at": "2026-04-01T12:00:00Z"
	}
}
```

Duplicate name behavior:

- Returns existing profile
- Status: `200`

```json
{
	"status": "success",
	"message": "Profile already exists",
	"data": {
		"id": "b3f9c1e2-7d4a-4c91-9c2a-1f0a8e5b6d12",
		"name": "ella",
		"gender": "female",
		"gender_probability": 0.99,
		"sample_size": 1234,
		"age": 46,
		"age_group": "adult",
		"country_id": "DRC",
		"country_probability": 0.85,
		"created_at": "2026-04-01T12:00:00Z"
	}
}
```

### 2) Get Single Profile

`GET /api/profiles/{id}`

Success: `200`

```json
{
	"status": "success",
	"data": {
		"id": "b3f9c1e2-7d4a-4c91-9c2a-1f0a8e5b6d12",
		"name": "emmanuel",
		"gender": "male",
		"gender_probability": 0.99,
		"sample_size": 1234,
		"age": 25,
		"age_group": "adult",
		"country_id": "NG",
		"country_probability": 0.85,
		"created_at": "2026-04-01T12:00:00Z"
	}
}
```

### 3) Get All Profiles

`GET /api/profiles`

Optional query parameters (case-insensitive):

- `gender`
- `country_id`
- `age_group`

Example:

`GET /api/profiles?gender=male&country_id=NG`

Success: `200`

```json
{
	"status": "success",
	"count": 2,
	"data": [
		{
			"id": "id-1",
			"name": "emmanuel",
			"gender": "male",
			"age": 25,
			"age_group": "adult",
			"country_id": "NG"
		},
		{
			"id": "id-2",
			"name": "sarah",
			"gender": "female",
			"age": 28,
			"age_group": "adult",
			"country_id": "US"
		}
	]
}
```

### 4) Delete Profile

`DELETE /api/profiles/{id}`

Success: `204 No Content`

## Error Format

All errors follow:

```json
{
	"status": "error",
	"message": "<error message>"
}
```

Common statuses:

- `400 Bad Request`: Missing or empty name
- `422 Unprocessable Entity`: Invalid type
- `404 Not Found`: Profile not found
- `500 Internal Server Error`: Server failure
- `502 Bad Gateway`: External API invalid response

`502` message format:

```json
{
	"status": "error",
	"message": "${externalApi} returned an invalid response"
}
```

Where `externalApi` is one of:

- `Genderize`
- `Agify`
- `Nationalize`

## External API Edge Cases

The service returns `502` and does not store data when:

- Genderize returns `gender: null`
- Genderize returns `count: 0`
- Agify returns `age: null`
- Nationalize returns no country data

## Graceful Shutdown

The server exits gracefully on:

- `Ctrl+C`
- `SIGTERM` (Unix)

This is implemented with Tokio signal handling and Axum graceful shutdown.

## Quick cURL Examples

Create:

```bash
curl -X POST http://localhost:3000/api/profiles \
	-H "Content-Type: application/json" \
	-d '{"name":"ella"}'
```

List:

```bash
curl "http://localhost:3000/api/profiles?gender=female"
```

Get by ID:

```bash
curl http://localhost:3000/api/profiles/<uuid>
```

Delete:

```bash
curl -X DELETE http://localhost:3000/api/profiles/<uuid>
```

## Development Commands

```bash
cargo fmt --all
cargo check
```
