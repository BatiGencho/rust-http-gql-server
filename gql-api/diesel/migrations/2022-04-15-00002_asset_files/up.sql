
CREATE TABLE IF NOT EXISTS asset_files (
    id UUID PRIMARY KEY,
    s3_bucket VARCHAR NOT NULL,
    s3_absolute_key VARCHAR NOT NULL,
    ipfs_hash VARCHAR NULL,
    event_id UUID REFERENCES events (id),
    UNIQUE (s3_bucket, s3_absolute_key)
)
