CREATE TABLE cloud_blobs (
    organization_id TEXT NOT NULL,
    project_id TEXT NOT NULL,
    digest TEXT NOT NULL,
    bytes INTEGER NOT NULL CHECK(bytes >= 0),
    PRIMARY KEY(organization_id,project_id,digest),
    FOREIGN KEY(organization_id,project_id) REFERENCES projects(organization_id,id)
);
CREATE TABLE project_snapshots (
    organization_id TEXT NOT NULL,
    project_id TEXT NOT NULL,
    revision INTEGER NOT NULL CHECK(revision > 0),
    manifest_json TEXT NOT NULL,
    PRIMARY KEY(organization_id,project_id,revision),
    FOREIGN KEY(organization_id,project_id) REFERENCES projects(organization_id,id)
);
CREATE TABLE project_heads (
    organization_id TEXT NOT NULL,
    project_id TEXT NOT NULL,
    revision INTEGER NOT NULL,
    PRIMARY KEY(organization_id,project_id),
    FOREIGN KEY(organization_id,project_id,revision) REFERENCES project_snapshots(organization_id,project_id,revision)
);
CREATE TABLE snapshot_blobs (
    organization_id TEXT NOT NULL,
    project_id TEXT NOT NULL,
    revision INTEGER NOT NULL,
    digest TEXT NOT NULL,
    PRIMARY KEY(organization_id,project_id,revision,digest),
    FOREIGN KEY(organization_id,project_id,revision) REFERENCES project_snapshots(organization_id,project_id,revision),
    FOREIGN KEY(organization_id,project_id,digest) REFERENCES cloud_blobs(organization_id,project_id,digest)
);
PRAGMA user_version = 3;
