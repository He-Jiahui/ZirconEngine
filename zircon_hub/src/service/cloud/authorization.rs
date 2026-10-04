use crate::service::{error::ServiceError, identity::Principal};
use rusqlite::{params, Connection, OptionalExtension};

pub(super) fn authorize(
    connection: &Connection,
    principal: &Principal,
    organization: &str,
    project: &str,
    write: bool,
) -> Result<i64, ServiceError> {
    let membership: Option<(String, i64)> = connection
        .query_row(
            "SELECT m.role,o.policy_revision FROM memberships m
             JOIN projects p ON p.organization_id=m.organization_id
             JOIN organizations o ON o.id=m.organization_id
             WHERE m.organization_id=?1 AND p.id=?2 AND m.issuer=?3 AND m.subject=?4 AND m.active=1",
            params![organization, project, principal.issuer, principal.subject],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    match membership {
        Some((role, revision))
            if matches!(role.as_str(), "owner" | "admin" | "member")
                || (!write && role == "viewer") =>
        {
            Ok(revision)
        }
        _ => Err(ServiceError::Forbidden),
    }
}
