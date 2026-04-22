CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE OR REPLACE FUNCTION uuid_generate_v7()
RETURNS UUID
LANGUAGE plpgsql
AS $$
DECLARE
    unix_ts_ms BIGINT;
    ts_hex TEXT;
    rand_hex TEXT;
    variant_nibble TEXT;
    uuid_hex TEXT;
BEGIN
    unix_ts_ms := FLOOR(EXTRACT(EPOCH FROM clock_timestamp()) * 1000);
    ts_hex := LPAD(TO_HEX(unix_ts_ms), 12, '0');
    rand_hex := ENCODE(gen_random_bytes(10), 'hex');
    variant_nibble := SUBSTR('89ab', (GET_BYTE(gen_random_bytes(1), 0) % 4) + 1, 1);

    uuid_hex := ts_hex
        || '7'
        || SUBSTR(rand_hex, 1, 3)
        || variant_nibble
        || SUBSTR(rand_hex, 4, 15);

    RETURN (
        SUBSTR(uuid_hex, 1, 8)
        || '-'
        || SUBSTR(uuid_hex, 9, 4)
        || '-'
        || SUBSTR(uuid_hex, 13, 4)
        || '-'
        || SUBSTR(uuid_hex, 17, 4)
        || '-'
        || SUBSTR(uuid_hex, 21, 12)
    )::UUID;
END;
$$;

ALTER TABLE profiles
    ALTER COLUMN id SET DEFAULT uuid_generate_v7();
