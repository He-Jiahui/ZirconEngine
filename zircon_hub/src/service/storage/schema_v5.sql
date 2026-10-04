CREATE TABLE cloud_store_binding (
    id INTEGER PRIMARY KEY CHECK(id = 1),
    store_id TEXT NOT NULL CHECK(length(store_id) = 36),
    key_fingerprint BLOB NOT NULL CHECK(typeof(key_fingerprint) = 'blob' AND length(key_fingerprint) = 32)
);
-- Existing v4 stores bind only after successful CAS recovery; no path or key is guessed here.
PRAGMA user_version = 5;
