use super::{now_seconds, params, OptionalExtension, Principal, ServiceError, Transaction};

pub(super) fn authorized_role(
    transaction: &Transaction<'_>,
    principal: &Principal,
    organization: &str,
    invitation: &str,
    replay: bool,
) -> Result<Option<String>, ServiceError> {
    // Recheck the qualified issuer in the same write transaction that grants membership.
    // An accepted retry additionally requires the recipient's membership to remain active.
    transaction
        .query_row(
            "SELECT i.role FROM invitations i
         JOIN memberships sender ON sender.organization_id=i.organization_id
             AND sender.issuer=i.inviter_issuer AND sender.subject=i.inviter_subject
         WHERE i.organization_id=?1 AND i.id=?2 AND i.target_issuer=?3 AND i.target_subject=?4
             AND sender.active=1 AND sender.role IN ('owner','admin')
             AND ((?6=0 AND i.status='pending' AND i.expires_at>?5)
                 OR (?6=1 AND i.status='accepted' AND EXISTS (
                     SELECT 1 FROM memberships recipient
                     WHERE recipient.organization_id=i.organization_id
                         AND recipient.issuer=i.target_issuer AND recipient.subject=i.target_subject
                         AND recipient.active=1)))",
            params![
                organization,
                invitation,
                principal.issuer,
                principal.subject,
                now_seconds(),
                replay
            ],
            |row| row.get(0),
        )
        .optional()
        .map_err(ServiceError::from)
}

pub(super) fn revoke_issued(
    transaction: &Transaction<'_>,
    organization: &str,
    issuer: &str,
    subject: &str,
) -> Result<(), ServiceError> {
    // Permanent status prevents old invitations becoming valid if authority is later restored.
    transaction.execute(
        "UPDATE invitations SET status='revoked'
         WHERE organization_id=?1 AND inviter_issuer=?2 AND inviter_subject=?3 AND status='pending'",
        params![organization, issuer, subject],
    )?;
    Ok(())
}
