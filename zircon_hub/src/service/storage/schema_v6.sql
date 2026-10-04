CREATE TABLE cloud_retention_policies (
    organization_id TEXT NOT NULL,
    project_id TEXT NOT NULL,
    revision INTEGER NOT NULL CHECK(revision > 0),
    keep_latest INTEGER CHECK(keep_latest BETWEEN 1 AND 128),
    trash_seconds INTEGER NOT NULL CHECK(trash_seconds BETWEEN 0 AND 2592000),
    legal_hold INTEGER NOT NULL CHECK(legal_hold IN (0,1)),
    PRIMARY KEY(organization_id,project_id),
    FOREIGN KEY(organization_id,project_id) REFERENCES projects(organization_id,id)
);
CREATE TABLE cloud_snapshot_trash (
    organization_id TEXT NOT NULL,
    project_id TEXT NOT NULL,
    revision INTEGER NOT NULL,
    purge_after INTEGER NOT NULL,
    PRIMARY KEY(organization_id,project_id,revision),
    FOREIGN KEY(organization_id,project_id,revision) REFERENCES project_snapshots(organization_id,project_id,revision)
);
CREATE TABLE cloud_upload_leases (
    organization_id TEXT NOT NULL,
    project_id TEXT NOT NULL,
    digest TEXT NOT NULL,
    expires_at INTEGER NOT NULL,
    PRIMARY KEY(organization_id,project_id,digest),
    FOREIGN KEY(organization_id,project_id,digest) REFERENCES cloud_blobs(organization_id,project_id,digest)
);
CREATE TABLE cloud_gc_pending (
    digest TEXT PRIMARY KEY,
    bytes INTEGER NOT NULL CHECK(bytes >= 0),
    organization_id TEXT NOT NULL,
    project_id TEXT NOT NULL
);
CREATE INDEX cloud_blobs_digest ON cloud_blobs(digest);
CREATE INDEX snapshot_blobs_digest ON snapshot_blobs(digest);
CREATE INDEX cloud_upload_leases_digest ON cloud_upload_leases(digest);
CREATE TABLE cloud_maintenance_runs (
    issuer TEXT NOT NULL,
    subject TEXT NOT NULL,
    operation_id TEXT NOT NULL,
    fingerprint TEXT NOT NULL,
    pruned_revisions INTEGER NOT NULL CHECK(pruned_revisions >= 0),
    PRIMARY KEY(issuer,subject,operation_id)
);
CREATE TRIGGER maintenance_operation_reserved BEFORE INSERT ON operation_receipts
WHEN EXISTS(SELECT 1 FROM cloud_maintenance_runs
            WHERE issuer=NEW.issuer AND subject=NEW.subject AND operation_id=NEW.operation_id
            AND fingerprint<>NEW.fingerprint)
BEGIN SELECT RAISE(ABORT, 'operation reserved'); END;
-- Preserve pre-migration uploads for a full upload lease. No retention policy is enabled here.
INSERT INTO cloud_upload_leases
    SELECT organization_id,project_id,digest,CAST(strftime('%s','now') AS INTEGER) + 86400
    FROM cloud_blobs;
PRAGMA user_version = 6;
