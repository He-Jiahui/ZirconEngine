CREATE TABLE organizations (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    policy_revision INTEGER NOT NULL CHECK(policy_revision > 0)
);
CREATE TABLE memberships (
    organization_id TEXT NOT NULL REFERENCES organizations(id),
    issuer TEXT NOT NULL,
    subject TEXT NOT NULL,
    role TEXT NOT NULL CHECK(role IN ('owner', 'admin', 'member', 'viewer')),
    active INTEGER NOT NULL CHECK(active IN (0, 1)),
    PRIMARY KEY (organization_id, issuer, subject)
);
CREATE TABLE projects (
    organization_id TEXT NOT NULL REFERENCES organizations(id),
    id TEXT NOT NULL,
    name TEXT NOT NULL,
    PRIMARY KEY(organization_id, id)
);
CREATE TABLE invitations (
    organization_id TEXT NOT NULL REFERENCES organizations(id),
    id TEXT NOT NULL,
    target_issuer TEXT NOT NULL,
    target_subject TEXT NOT NULL,
    role TEXT NOT NULL CHECK(role IN ('admin', 'member', 'viewer')),
    expires_at INTEGER NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('pending','accepted','revoked')),
    PRIMARY KEY(organization_id, id)
);
CREATE TABLE audit_events (
    sequence INTEGER PRIMARY KEY AUTOINCREMENT,
    organization_id TEXT NOT NULL REFERENCES organizations(id),
    actor_digest TEXT NOT NULL,
    action TEXT NOT NULL,
    outcome TEXT NOT NULL,
    policy_revision INTEGER NOT NULL,
    occurred_at INTEGER NOT NULL
);
CREATE UNIQUE INDEX invitation_id ON invitations(id);
CREATE INDEX invitation_target ON invitations(target_issuer,target_subject,id);
CREATE INDEX membership_principal ON memberships(issuer,subject,active,organization_id);
CREATE TRIGGER audit_no_update BEFORE UPDATE ON audit_events BEGIN SELECT RAISE(ABORT, 'immutable audit'); END;
CREATE TRIGGER audit_no_delete BEFORE DELETE ON audit_events BEGIN SELECT RAISE(ABORT, 'immutable audit'); END;
CREATE TABLE operation_receipts (
    issuer TEXT NOT NULL,
    subject TEXT NOT NULL,
    operation_id TEXT NOT NULL,
    fingerprint TEXT NOT NULL,
    result_json TEXT NOT NULL,
    committed_at INTEGER NOT NULL,
    PRIMARY KEY (issuer, subject, operation_id)
);
PRAGMA user_version = 1;
