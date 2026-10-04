CREATE TABLE catalog_releases (
    package_id TEXT NOT NULL,
    revision INTEGER NOT NULL CHECK(revision > 0),
    publisher_issuer TEXT NOT NULL,
    publisher_subject TEXT NOT NULL,
    key_id TEXT NOT NULL,
    name TEXT NOT NULL,
    kind TEXT NOT NULL CHECK(kind IN ('plugin','asset')),
    license_id TEXT NOT NULL,
    digest TEXT NOT NULL,
    expires_at INTEGER NOT NULL,
    envelope TEXT NOT NULL,
    PRIMARY KEY(package_id, revision)
);
CREATE TABLE catalog_heads (
    package_id TEXT PRIMARY KEY,
    revision INTEGER NOT NULL,
    FOREIGN KEY(package_id,revision) REFERENCES catalog_releases(package_id,revision)
);
CREATE TABLE entitlements (
    organization_id TEXT NOT NULL REFERENCES organizations(id),
    package_id TEXT NOT NULL,
    revision INTEGER NOT NULL,
    license_id TEXT NOT NULL,
    accepted_by_issuer TEXT NOT NULL,
    accepted_by_subject TEXT NOT NULL,
    accepted_at INTEGER NOT NULL,
    PRIMARY KEY(organization_id,package_id,revision),
    FOREIGN KEY(package_id,revision) REFERENCES catalog_releases(package_id,revision)
);
CREATE TRIGGER catalog_no_update BEFORE UPDATE ON catalog_releases BEGIN SELECT RAISE(ABORT, 'immutable release'); END;
CREATE TRIGGER catalog_no_delete BEFORE DELETE ON catalog_releases BEGIN SELECT RAISE(ABORT, 'immutable release'); END;
CREATE INDEX catalog_listing ON catalog_releases(kind,name,package_id);
CREATE TABLE catalog_audit (
    sequence INTEGER PRIMARY KEY AUTOINCREMENT,
    package_id TEXT NOT NULL,
    revision INTEGER NOT NULL,
    actor_digest TEXT NOT NULL,
    occurred_at INTEGER NOT NULL,
    FOREIGN KEY(package_id,revision) REFERENCES catalog_releases(package_id,revision)
);
CREATE TRIGGER catalog_audit_no_update BEFORE UPDATE ON catalog_audit BEGIN SELECT RAISE(ABORT, 'immutable catalog audit'); END;
CREATE TRIGGER catalog_audit_no_delete BEFORE DELETE ON catalog_audit BEGIN SELECT RAISE(ABORT, 'immutable catalog audit'); END;
PRAGMA user_version = 2;
