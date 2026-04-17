CREATE TABLE IF NOT EXISTS profiles (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    gender TEXT NOT NULL,
    gender_probability DOUBLE PRECISION NOT NULL,
    sample_size BIGINT NOT NULL,
    age INTEGER NOT NULL,
    age_group TEXT NOT NULL,
    country_id TEXT NOT NULL,
    country_probability DOUBLE PRECISION NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS profiles_gender_idx ON profiles (gender);
CREATE INDEX IF NOT EXISTS profiles_country_id_idx ON profiles (country_id);
CREATE INDEX IF NOT EXISTS profiles_age_group_idx ON profiles (age_group);