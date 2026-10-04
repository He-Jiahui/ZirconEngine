use super::{
    config::read_bounded,
    error::ServiceError,
    identity::{now_seconds, Principal},
    storage::receipt,
};
use jsonwebtoken::{decode, decode_header, jwk::JwkSet, Algorithm, DecodingKey, Validation};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};

pub mod artifacts;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Publisher {
    pub issuer: String,
    pub subject: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogPolicy {
    issuer: String,
    audience: String,
    keys: JwkSet,
    publishers: BTreeMap<String, Publisher>,
}

impl CatalogPolicy {
    pub fn load(path: Option<&Path>) -> Result<Self, ServiceError> {
        let policy: Self = serde_json::from_slice(&read_bounded(
            path.ok_or(ServiceError::Configuration)?,
            65536,
        )?)
        .map_err(|_| ServiceError::Configuration)?;
        if policy.keys.keys.is_empty()
            || policy.keys.keys.len() > 32
            || policy.publishers.len() > 32
            || policy.issuer.is_empty()
            || policy.audience.is_empty()
        {
            return Err(ServiceError::Configuration);
        }
        Ok(policy)
    }

    fn verify(&self, envelope: &str) -> Result<(String, Release), ServiceError> {
        if envelope.len() > 32768 {
            return Err(ServiceError::InvalidRequest);
        }
        let header = decode_header(envelope).map_err(|_| ServiceError::InvalidRequest)?;
        if header.alg != Algorithm::RS256 {
            return Err(ServiceError::Forbidden);
        }
        let kid = header.kid.ok_or(ServiceError::Forbidden)?;
        if !self.publishers.contains_key(&kid) {
            return Err(ServiceError::Forbidden);
        }
        let key = self.keys.find(&kid).ok_or(ServiceError::Forbidden)?;
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&[&self.audience]);
        validation.leeway = 0;
        validation.set_required_spec_claims(&["iss", "aud", "exp", "sub"]);
        let release = decode::<Release>(
            envelope,
            &DecodingKey::from_jwk(key).map_err(|_| ServiceError::Forbidden)?,
            &validation,
        )
        .map_err(|_| ServiceError::Forbidden)?
        .claims;
        receipt::validate_id(&release.package_id)?;
        if release.exp <= now_seconds()
            || release.revision <= 0
            || release.name.is_empty()
            || release.name.len() > 256
            || release.description.len() > 8192
            || release.license_id.is_empty()
            || release.license_id.len() > 128
            || release.license_text.len() > 8192
            || release.artifact_digest.len() != 64
            || !release
                .artifact_digest
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            || release.artifact_size == 0
            || release.artifact_size > 1073741824
            || !matches!(release.kind.as_str(), "plugin" | "asset")
            || release.version.is_empty()
            || release.version.len() > 64
        {
            return Err(ServiceError::InvalidRequest);
        }
        if self.publishers[&kid].subject != release.sub {
            return Err(ServiceError::Forbidden);
        }
        Ok((kid, release))
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Release {
    pub iss: String,
    pub aud: String,
    pub sub: String,
    pub exp: u64,
    pub package_id: String,
    #[serde(with = "revision_string")]
    pub revision: i64,
    pub version: String,
    pub name: String,
    pub kind: String,
    pub description: String,
    pub license_id: String,
    pub license_text: String,
    pub artifact_digest: String,
    pub artifact_size: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PublishRequest {
    pub operation_id: String,
    pub envelope: String,
}

mod revision_string {
    use serde::{Deserialize, Deserializer, Serializer};
    pub fn serialize<S: Serializer>(value: &i64, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&value.to_string())
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<i64, D::Error> {
        let value = String::deserialize(deserializer)?;
        let revision = value.parse::<i64>().map_err(serde::de::Error::custom)?;
        if revision <= 0 || revision.to_string() != value {
            return Err(serde::de::Error::custom("invalid release revision"));
        }
        Ok(revision)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Publication {
    pub package_id: String,
    pub revision: String,
}

pub fn publish(
    connection: &mut Connection,
    principal: &Principal,
    policy: &CatalogPolicy,
    request: PublishRequest,
) -> Result<Publication, ServiceError> {
    let (kid, release) = policy.verify(&request.envelope)?;
    let publisher = &policy.publishers[&kid];
    if publisher.issuer != principal.issuer || publisher.subject != principal.subject {
        return Err(ServiceError::Forbidden);
    }
    let fingerprint = receipt::fingerprint(&("catalog.publish", &request.envelope))?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    if let Some(result) =
        receipt::replay(&transaction, principal, &request.operation_id, &fingerprint)?
    {
        return Ok(result);
    }
    let head: Option<(i64, String, String)> = transaction.query_row("SELECT r.revision,r.publisher_issuer,r.publisher_subject FROM catalog_heads h JOIN catalog_releases r ON r.package_id=h.package_id AND r.revision=h.revision WHERE h.package_id=?1", [&release.package_id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))).optional()?;
    if let Some((revision, issuer, subject)) = head {
        if revision >= release.revision
            || issuer != principal.issuer
            || subject != principal.subject
        {
            return Err(ServiceError::Conflict);
        }
    }
    transaction.execute(
        "INSERT INTO catalog_releases VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
        params![
            release.package_id,
            release.revision,
            principal.issuer,
            principal.subject,
            kid,
            release.name,
            release.kind,
            release.license_id,
            release.artifact_digest,
            release.exp,
            request.envelope
        ],
    )?;
    transaction.execute("INSERT INTO catalog_heads VALUES (?1,?2) ON CONFLICT(package_id) DO UPDATE SET revision=excluded.revision", params![release.package_id, release.revision])?;
    let actor_digest = receipt::fingerprint(&(&principal.issuer, &principal.subject))?;
    transaction.execute("INSERT INTO catalog_audit(package_id,revision,actor_digest,occurred_at) VALUES (?1,?2,?3,?4)", params![release.package_id, release.revision, actor_digest, now_seconds()])?;
    let result = Publication {
        package_id: release.package_id,
        revision: release.revision.to_string(),
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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CatalogQuery {
    pub after: Option<String>,
    pub query: Option<String>,
    pub limit: Option<u16>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogPage {
    pub items: Vec<Release>,
    pub next_cursor: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Entitlement {
    pub package_id: String,
    pub revision: String,
    pub license_id: String,
}

pub fn entitlements(
    connection: &Connection,
    principal: &Principal,
    organization: &str,
    after: Option<String>,
) -> Result<super::organization::Page<Entitlement>, ServiceError> {
    let allowed: bool = connection.query_row("SELECT EXISTS(SELECT 1 FROM memberships WHERE organization_id=?1 AND issuer=?2 AND subject=?3 AND active=1)", params![organization, principal.issuer, principal.subject], |row| row.get(0))?;
    if !allowed {
        return Err(ServiceError::Forbidden);
    }
    let after = after
        .unwrap_or_else(|| "0".into())
        .parse::<i64>()
        .map_err(|_| ServiceError::InvalidRequest)?;
    if after < 0 {
        return Err(ServiceError::InvalidRequest);
    }
    let mut statement = connection.prepare("SELECT rowid,package_id,revision,license_id FROM entitlements WHERE organization_id=?1 AND rowid>?2 ORDER BY rowid LIMIT 51")?;
    let rows = statement.query_map(params![organization, after], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            Entitlement {
                package_id: row.get(1)?,
                revision: row.get::<_, i64>(2)?.to_string(),
                license_id: row.get(3)?,
            },
        ))
    })?;
    let mut rows = rows.collect::<Result<Vec<_>, _>>()?;
    let more = rows.len() > 50;
    rows.truncate(50);
    let next_cursor = more.then(|| rows.last().unwrap().0.to_string());
    Ok(super::organization::Page {
        items: rows.into_iter().map(|(_, item)| item).collect(),
        next_cursor,
    })
}

pub fn authorized_artifact(
    connection: &Connection,
    principal: &Principal,
    policy: &CatalogPolicy,
    organization: &str,
    package: &str,
    revision: &str,
) -> Result<Release, ServiceError> {
    let revision = revision
        .parse::<i64>()
        .map_err(|_| ServiceError::InvalidRequest)?;
    let envelope: Option<String> = connection.query_row("SELECT r.envelope FROM entitlements e JOIN memberships m ON m.organization_id=e.organization_id JOIN catalog_releases r ON r.package_id=e.package_id AND r.revision=e.revision WHERE e.organization_id=?1 AND e.package_id=?2 AND e.revision=?3 AND m.issuer=?4 AND m.subject=?5 AND m.active=1", params![organization, package, revision, principal.issuer, principal.subject], |row| row.get(0)).optional()?;
    policy
        .verify(&envelope.ok_or(ServiceError::Forbidden)?)
        .map(|(_, release)| release)
}

pub fn list(
    connection: &Connection,
    policy: &CatalogPolicy,
    query: CatalogQuery,
) -> Result<CatalogPage, ServiceError> {
    let after = query.after.unwrap_or_default();
    if !after.is_empty() {
        receipt::validate_id(&after)?;
    }
    let search = query.query.unwrap_or_default();
    let limit = usize::from(query.limit.unwrap_or(50));
    if search.len() > 256 || !(1..=100).contains(&limit) {
        return Err(ServiceError::InvalidRequest);
    }
    let mut statement = connection.prepare("SELECT r.package_id,r.envelope FROM catalog_heads h JOIN catalog_releases r ON r.package_id=h.package_id AND r.revision=h.revision WHERE r.package_id>?1 AND instr(lower(r.name),lower(?2))>0 ORDER BY r.package_id LIMIT ?3")?;
    let rows = statement.query_map(params![after, search, limit + 1], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    let mut rows = rows.collect::<Result<Vec<_>, _>>()?;
    let more = rows.len() > limit;
    rows.truncate(limit);
    let next_cursor = more.then(|| rows.last().unwrap().0.clone());
    let items = rows
        .into_iter()
        .filter_map(|(_, envelope)| policy.verify(&envelope).ok().map(|(_, release)| release))
        .collect();
    Ok(CatalogPage { items, next_cursor })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AcceptLicense {
    pub operation_id: String,
    pub expected_policy_revision: String,
    pub package_id: String,
    pub revision: String,
    pub license_id: String,
}

pub fn accept_license(
    connection: &mut Connection,
    principal: &Principal,
    policy: &CatalogPolicy,
    organization: &str,
    request: AcceptLicense,
) -> Result<Publication, ServiceError> {
    let revision = request
        .revision
        .parse::<i64>()
        .map_err(|_| ServiceError::InvalidRequest)?;
    if revision <= 0 || revision.to_string() != request.revision {
        return Err(ServiceError::InvalidRequest);
    }
    let fingerprint = receipt::fingerprint(&(
        "catalog.license",
        organization,
        &request.package_id,
        &request.revision,
        &request.license_id,
        &request.expected_policy_revision,
    ))?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    if let Some(result) =
        receipt::replay(&transaction, principal, &request.operation_id, &fingerprint)?
    {
        return Ok(result);
    }
    let policy_revision: Option<i64> = transaction.query_row("SELECT o.policy_revision FROM organizations o JOIN memberships m ON m.organization_id=o.id WHERE o.id=?1 AND m.issuer=?2 AND m.subject=?3 AND m.active=1 AND m.role IN ('owner','admin')", params![organization, principal.issuer, principal.subject], |row| row.get(0)).optional()?;
    let current = policy_revision.ok_or(ServiceError::Forbidden)?;
    if current.to_string() != request.expected_policy_revision {
        return Err(ServiceError::Conflict);
    }
    let envelope: Option<String> = transaction.query_row("SELECT r.envelope FROM catalog_heads h JOIN catalog_releases r ON r.package_id=h.package_id AND r.revision=h.revision WHERE h.package_id=?1 AND h.revision=?2", params![request.package_id, revision], |row| row.get(0)).optional()?;
    let (_, release) = policy.verify(&envelope.ok_or(ServiceError::Conflict)?)?;
    if release.license_id != request.license_id {
        return Err(ServiceError::Conflict);
    }
    transaction.execute("INSERT INTO entitlements VALUES (?1,?2,?3,?4,?5,?6,?7) ON CONFLICT(organization_id,package_id,revision) DO NOTHING", params![organization, request.package_id, revision, request.license_id, principal.issuer, principal.subject, now_seconds()])?;
    super::organization::audit(
        &transaction,
        organization,
        principal,
        "license.accept",
        "allowed",
        current,
    )?;
    let result = Publication {
        package_id: request.package_id,
        revision: request.revision,
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

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
