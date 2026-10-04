CREATE TABLE catalog_artifacts (
    digest TEXT PRIMARY KEY CHECK(length(digest) = 64),
    payload BLOB NOT NULL CHECK(length(payload) BETWEEN 1 AND 16777216)
);
PRAGMA user_version = 7;
