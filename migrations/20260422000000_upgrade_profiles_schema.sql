CREATE EXTENSION IF NOT EXISTS pgcrypto;

ALTER TABLE profiles
    ADD COLUMN IF NOT EXISTS country_name VARCHAR NOT NULL DEFAULT '',
    ALTER COLUMN id SET DEFAULT gen_random_uuid();

UPDATE profiles
SET country_name = country_id
WHERE country_name = '';

ALTER TABLE profiles
    ALTER COLUMN name TYPE VARCHAR,
    ALTER COLUMN gender TYPE VARCHAR,
    ALTER COLUMN country_id TYPE VARCHAR(2),
    ALTER COLUMN country_name TYPE VARCHAR,
    ALTER COLUMN created_at TYPE TIMESTAMP USING created_at;

ALTER TABLE profiles
    DROP COLUMN IF EXISTS sample_size;

CREATE INDEX IF NOT EXISTS profiles_age_idx ON profiles (age);
CREATE INDEX IF NOT EXISTS profiles_created_at_idx ON profiles (created_at DESC);
CREATE INDEX IF NOT EXISTS profiles_gender_probability_idx ON profiles (gender_probability);
CREATE INDEX IF NOT EXISTS profiles_country_probability_idx ON profiles (country_probability);
CREATE INDEX IF NOT EXISTS profiles_gender_country_age_idx ON profiles (gender, country_id, age);
CREATE INDEX IF NOT EXISTS profiles_country_name_lower_idx ON profiles ((LOWER(country_name)));
