use super::{
    error::ServiceError,
    identity::{now_seconds, Principal},
    storage::receipt,
};
use rusqlite::{params, Connection, OptionalExtension, Transaction, TransactionBehavior};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
mod invitation;
mod query;
pub use query::{
    invitations, issued_invitations, list, members, projects, Invitation, IssuedInvitation, Member,
    Page, PageQuery, Project,
};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Organization {
    pub id: String,
    pub name: String,
    pub policy_revision: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateOrganization {
    pub operation_id: String,
    pub name: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Mutation {
    CreateProject {
        name: String,
    },
    SetMember {
        issuer: String,
        subject: String,
        role: String,
        active: bool,
    },
    TransferOwnership {
        issuer: String,
        subject: String,
    },
    Invite {
        issuer: String,
        subject: String,
        role: String,
        expires_at: u64,
    },
    AcceptInvite {
        invitation_id: String,
    },
    RevokeInvite {
        invitation_id: String,
    },
}

impl Mutation {
    fn action(&self) -> &'static str {
        match self {
            Self::CreateProject { .. } => "project.create",
            Self::SetMember { .. } => "member.update",
            Self::TransferOwnership { .. } => "owner.transfer",
            Self::Invite { .. } => "invite.create",
            Self::AcceptInvite { .. } => "invite.accept",
            Self::RevokeInvite { .. } => "invite.revoke",
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MutationRequest {
    pub operation_id: String,
    pub expected_policy_revision: String,
    pub mutation: Mutation,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Receipt {
    pub organization_id: String,
    pub resource_id: String,
    pub policy_revision: String,
    pub audit_sequence: i64,
}

pub fn create(
    connection: &mut Connection,
    principal: &Principal,
    operation_id: &str,
    name: &str,
) -> Result<Organization, ServiceError> {
    validate_text(name)?;
    let fingerprint = receipt::fingerprint(&("organization.create", name))?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    if let Some(result) = receipt::replay(&transaction, principal, operation_id, &fingerprint)? {
        return Ok(result);
    }
    let id = uuid::Uuid::new_v4().to_string();
    transaction.execute(
        "INSERT INTO organizations VALUES (?1, ?2, 1)",
        params![id, name],
    )?;
    transaction.execute(
        "INSERT INTO memberships VALUES (?1, ?2, ?3, 'owner', 1)",
        params![id, principal.issuer, principal.subject],
    )?;
    audit(
        &transaction,
        &id,
        principal,
        "organization.create",
        "allowed",
        1,
    )?;
    let result = Organization {
        id,
        name: name.to_owned(),
        policy_revision: "1".to_owned(),
    };
    receipt::commit(&transaction, principal, operation_id, &fingerprint, &result)?;
    transaction.commit()?;
    Ok(result)
}

pub fn mutate(
    connection: &mut Connection,
    principal: &Principal,
    organization_id: &str,
    request: MutationRequest,
) -> Result<Receipt, ServiceError> {
    let fingerprint = receipt::fingerprint(&("organization.mutate", organization_id, &request))?;
    let expected = request
        .expected_policy_revision
        .parse::<i64>()
        .map_err(|_| ServiceError::InvalidRequest)?;
    if expected <= 0 || expected.to_string() != request.expected_policy_revision {
        return Err(ServiceError::InvalidRequest);
    }
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let accepting = matches!(request.mutation, Mutation::AcceptInvite { .. });
    let replayed: Option<Receipt> =
        receipt::replay(&transaction, principal, &request.operation_id, &fingerprint)?;
    if !accepting {
        if let Some(result) = replayed {
            return Ok(result);
        }
    }
    let revision: Option<i64> = transaction
        .query_row(
            "SELECT policy_revision FROM organizations WHERE id=?1",
            [organization_id],
            |row| row.get(0),
        )
        .optional()?;
    let Some(revision) = revision else {
        return Err(ServiceError::Forbidden);
    };
    let actor_role: Option<String> = transaction.query_row("SELECT role FROM memberships WHERE organization_id=?1 AND issuer=?2 AND subject=?3 AND active=1", params![organization_id, principal.issuer, principal.subject], |row| row.get(0)).optional()?;
    let invitation_role = if let Mutation::AcceptInvite { invitation_id } = &request.mutation {
        invitation::authorized_role(
            &transaction,
            principal,
            organization_id,
            invitation_id,
            replayed.is_some(),
        )?
    } else {
        None
    };
    let allowed = match &request.mutation {
        Mutation::CreateProject { .. } => {
            matches!(actor_role.as_deref(), Some("owner" | "admin" | "member"))
        }
        Mutation::SetMember { .. } => actor_role.as_deref() == Some("owner"),
        Mutation::TransferOwnership { .. } => actor_role.as_deref() == Some("owner"),
        Mutation::Invite { .. } | Mutation::RevokeInvite { .. } => {
            matches!(actor_role.as_deref(), Some("owner" | "admin"))
        }
        Mutation::AcceptInvite { .. } => invitation_role.is_some(),
    };
    if !allowed {
        audit(
            &transaction,
            organization_id,
            principal,
            request.mutation.action(),
            "denied",
            revision,
        )?;
        transaction.commit()?;
        return Err(ServiceError::Forbidden);
    }
    // Retry is read-only, but it cannot present a revoked invitation grant as current authority.
    // The principal-scoped operation lookup remains available for historical reconciliation.
    if let Some(result) = replayed {
        return Ok(result);
    }
    if revision != expected {
        return Err(ServiceError::Conflict);
    }
    let next_revision = revision.checked_add(1).ok_or(ServiceError::Conflict)?;
    let resource_id = match &request.mutation {
        Mutation::CreateProject { name } => {
            validate_text(name)?;
            let id = uuid::Uuid::new_v4().to_string();
            transaction.execute(
                "INSERT INTO projects VALUES (?1,?2,?3)",
                params![organization_id, id, name],
            )?;
            id
        }
        Mutation::SetMember {
            issuer,
            subject,
            role,
            active,
        } => {
            validate_identity(issuer, subject)?;
            validate_role(role)?;
            let existing: Option<String> = transaction.query_row("SELECT role FROM memberships WHERE organization_id=?1 AND issuer=?2 AND subject=?3", params![organization_id, issuer, subject], |row| row.get(0)).optional()?;
            // Ownership changes use TransferOwnership; member updates cannot remove an owner.
            if existing.as_deref() == Some("owner") || existing.is_none() {
                return Err(ServiceError::InvalidRequest);
            }
            transaction.execute("UPDATE memberships SET role=?4,active=?5 WHERE organization_id=?1 AND issuer=?2 AND subject=?3", params![organization_id, issuer, subject, role, active])?;
            transaction.execute("UPDATE invitations SET status='revoked' WHERE organization_id=?1 AND target_issuer=?2 AND target_subject=?3 AND status='pending'", params![organization_id, issuer, subject])?;
            if !active || role != "admin" {
                invitation::revoke_issued(&transaction, organization_id, issuer, subject)?;
            }
            subject.clone()
        }
        Mutation::TransferOwnership { issuer, subject } => {
            validate_identity(issuer, subject)?;
            if issuer == &principal.issuer && subject == &principal.subject {
                return Err(ServiceError::InvalidRequest);
            }
            let changed = transaction.execute("UPDATE memberships SET role='owner' WHERE organization_id=?1 AND issuer=?2 AND subject=?3 AND active=1", params![organization_id, issuer, subject])?;
            if changed != 1 {
                return Err(ServiceError::Forbidden);
            }
            transaction.execute("UPDATE memberships SET role='admin' WHERE organization_id=?1 AND issuer=?2 AND subject=?3", params![organization_id, principal.issuer, principal.subject])?;
            format!("{issuer}:{subject}")
        }
        Mutation::Invite {
            issuer,
            subject,
            role,
            expires_at,
        } => {
            validate_identity(issuer, subject)?;
            validate_role(role)?;
            let existing: bool = transaction.query_row("SELECT EXISTS(SELECT 1 FROM memberships WHERE organization_id=?1 AND issuer=?2 AND subject=?3)", params![organization_id, issuer, subject], |row| row.get(0))?;
            if existing {
                return Err(ServiceError::InvalidRequest);
            }
            let now = now_seconds();
            if *expires_at <= now || *expires_at > now + 604800 {
                return Err(ServiceError::InvalidRequest);
            }
            let id = uuid::Uuid::new_v4().to_string();
            transaction.execute(
                "INSERT INTO invitations (organization_id,id,target_issuer,target_subject,role,expires_at,status,inviter_issuer,inviter_subject) VALUES (?1,?2,?3,?4,?5,?6,'pending',?7,?8)",
                params![organization_id, id, issuer, subject, role, expires_at, principal.issuer, principal.subject],
            )?;
            id
        }
        Mutation::AcceptInvite { invitation_id } => {
            let role = invitation_role.ok_or(ServiceError::Forbidden)?;
            let existing: Option<String> = transaction.query_row("SELECT role FROM memberships WHERE organization_id=?1 AND issuer=?2 AND subject=?3", params![organization_id, principal.issuer, principal.subject], |row| row.get(0)).optional()?;
            if existing.is_some() {
                return Err(ServiceError::InvalidRequest);
            }
            transaction.execute(
                "INSERT INTO memberships VALUES (?1,?2,?3,?4,1)",
                params![organization_id, principal.issuer, principal.subject, role],
            )?;
            transaction.execute(
                "UPDATE invitations SET status='accepted' WHERE organization_id=?1 AND id=?2",
                params![organization_id, invitation_id],
            )?;
            invitation_id.clone()
        }
        Mutation::RevokeInvite { invitation_id } => {
            let changed = transaction.execute("UPDATE invitations SET status='revoked' WHERE organization_id=?1 AND id=?2 AND status='pending'", params![organization_id, invitation_id])?;
            if changed != 1 {
                return Err(ServiceError::Conflict);
            }
            invitation_id.clone()
        }
    };
    transaction.execute(
        "UPDATE organizations SET policy_revision=?2 WHERE id=?1",
        params![organization_id, next_revision],
    )?;
    let audit_sequence = audit(
        &transaction,
        organization_id,
        principal,
        request.mutation.action(),
        "allowed",
        next_revision,
    )?;
    let result = Receipt {
        organization_id: organization_id.to_owned(),
        resource_id,
        policy_revision: next_revision.to_string(),
        audit_sequence,
    };
    receipt::commit(
        &transaction,
        principal,
        &request.operation_id,
        &fingerprint,
        &result,
    )?;
    transaction.commit()?;
    Ok(result)
}

pub(super) fn audit(
    transaction: &Transaction<'_>,
    organization: &str,
    principal: &Principal,
    action: &str,
    outcome: &str,
    revision: i64,
) -> Result<i64, ServiceError> {
    let identity_bytes = serde_json::to_vec(&(&principal.issuer, &principal.subject))
        .map_err(|_| ServiceError::Storage)?;
    let digest = format!("{:x}", Sha256::digest(identity_bytes));
    transaction.execute("INSERT INTO audit_events (organization_id,actor_digest,action,outcome,policy_revision,occurred_at) VALUES (?1,?2,?3,?4,?5,?6)", params![organization, digest, action, outcome, revision, now_seconds()])?;
    Ok(transaction.last_insert_rowid())
}

fn validate_text(text: &str) -> Result<(), ServiceError> {
    if text.trim().is_empty() || text.len() > 256 || text.chars().any(char::is_control) {
        return Err(ServiceError::InvalidRequest);
    }
    Ok(())
}

fn validate_identity(issuer: &str, subject: &str) -> Result<(), ServiceError> {
    validate_text(issuer)?;
    validate_text(subject)
}

fn validate_role(role: &str) -> Result<(), ServiceError> {
    if !matches!(role, "admin" | "member" | "viewer") {
        return Err(ServiceError::InvalidRequest);
    }
    Ok(())
}

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
