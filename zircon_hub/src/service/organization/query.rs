use super::*;

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PageQuery {
    pub after: Option<String>,
    pub limit: Option<u16>,
}

impl PageQuery {
    fn bounds(&self) -> Result<(&str, usize), ServiceError> {
        if let Some(after) = &self.after {
            receipt::validate_id(after)?;
        }
        let limit = usize::from(self.limit.unwrap_or(50));
        if !(1..=100).contains(&limit) {
            return Err(ServiceError::InvalidRequest);
        }
        Ok((self.after.as_deref().unwrap_or(""), limit))
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Page<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Invitation {
    pub id: String,
    pub organization_id: String,
    pub organization_name: String,
    pub policy_revision: String,
    pub role: String,
    pub expires_at: u64,
    pub status: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IssuedInvitation {
    pub id: String,
    pub organization_id: String,
    pub policy_revision: String,
    pub target_issuer: String,
    pub target_subject: String,
    pub role: String,
    pub expires_at: u64,
    pub status: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Member {
    pub issuer: String,
    pub subject: String,
    pub role: String,
    pub active: bool,
}

#[derive(Serialize)]
pub struct Project {
    pub id: String,
    pub name: String,
}

fn require_member(
    connection: &Connection,
    principal: &Principal,
    organization: &str,
) -> Result<(), ServiceError> {
    let allowed: bool = connection.query_row("SELECT EXISTS(SELECT 1 FROM memberships WHERE organization_id=?1 AND issuer=?2 AND subject=?3 AND active=1)", params![organization, principal.issuer, principal.subject], |row| row.get(0))?;
    if !allowed {
        return Err(ServiceError::Forbidden);
    }
    Ok(())
}

pub fn members(
    connection: &Connection,
    principal: &Principal,
    organization: &str,
    query: PageQuery,
) -> Result<Page<Member>, ServiceError> {
    require_member(connection, principal, organization)?;
    let after: i64 = query
        .after
        .as_deref()
        .unwrap_or("0")
        .parse()
        .map_err(|_| ServiceError::InvalidRequest)?;
    let limit = usize::from(query.limit.unwrap_or(50));
    if after < 0 || !(1..=100).contains(&limit) {
        return Err(ServiceError::InvalidRequest);
    }
    let mut statement = connection.prepare("SELECT rowid,issuer,subject,role,active FROM memberships WHERE organization_id=?1 AND rowid>?2 ORDER BY rowid LIMIT ?3")?;
    let rows = statement.query_map(params![organization, after, limit + 1], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            Member {
                issuer: row.get(1)?,
                subject: row.get(2)?,
                role: row.get(3)?,
                active: row.get(4)?,
            },
        ))
    })?;
    let mut items = rows.collect::<Result<Vec<_>, _>>()?;
    let more = items.len() > limit;
    items.truncate(limit);
    let next_cursor = more.then(|| items.last().unwrap().0.to_string());
    Ok(Page {
        items: items.into_iter().map(|(_, member)| member).collect(),
        next_cursor,
    })
}

pub fn projects(
    connection: &Connection,
    principal: &Principal,
    organization: &str,
    query: PageQuery,
) -> Result<Page<Project>, ServiceError> {
    require_member(connection, principal, organization)?;
    let (after, limit) = query.bounds()?;
    let mut statement = connection.prepare(
        "SELECT id,name FROM projects WHERE organization_id=?1 AND id>?2 ORDER BY id LIMIT ?3",
    )?;
    let rows = statement.query_map(params![organization, after, limit + 1], |row| {
        Ok(Project {
            id: row.get(0)?,
            name: row.get(1)?,
        })
    })?;
    let mut items = rows.collect::<Result<Vec<_>, _>>()?;
    let more = items.len() > limit;
    items.truncate(limit);
    let next_cursor = more.then(|| items.last().unwrap().id.clone());
    Ok(Page { items, next_cursor })
}

pub fn list(
    connection: &Connection,
    principal: &Principal,
    query: PageQuery,
) -> Result<Page<Organization>, ServiceError> {
    let (after, limit) = query.bounds()?;
    let mut statement = connection.prepare("SELECT o.id,o.name,o.policy_revision FROM organizations o JOIN memberships m ON m.organization_id=o.id WHERE m.issuer=?1 AND m.subject=?2 AND m.active=1 AND o.id>?3 ORDER BY o.id LIMIT ?4")?;
    let rows = statement.query_map(
        params![principal.issuer, principal.subject, after, limit + 1],
        |row| {
            Ok(Organization {
                id: row.get(0)?,
                name: row.get(1)?,
                policy_revision: row.get::<_, i64>(2)?.to_string(),
            })
        },
    )?;
    let mut items = rows.collect::<Result<Vec<_>, _>>()?;
    let more = items.len() > limit;
    items.truncate(limit);
    let next_cursor = more.then(|| items.last().unwrap().id.clone());
    Ok(Page { items, next_cursor })
}

pub fn invitations(
    connection: &Connection,
    principal: &Principal,
    query: PageQuery,
) -> Result<Page<Invitation>, ServiceError> {
    let (after, limit) = query.bounds()?;
    let mut statement = connection.prepare("SELECT i.id,i.organization_id,o.name,o.policy_revision,i.role,i.expires_at,i.status FROM invitations i JOIN organizations o ON o.id=i.organization_id WHERE i.target_issuer=?1 AND i.target_subject=?2 AND i.id>?3 ORDER BY i.id LIMIT ?4")?;
    let rows = statement.query_map(
        params![principal.issuer, principal.subject, after, limit + 1],
        |row| {
            Ok(Invitation {
                id: row.get(0)?,
                organization_id: row.get(1)?,
                organization_name: row.get(2)?,
                policy_revision: row.get::<_, i64>(3)?.to_string(),
                role: row.get(4)?,
                expires_at: row.get(5)?,
                status: row.get(6)?,
            })
        },
    )?;
    let mut items = rows.collect::<Result<Vec<_>, _>>()?;
    let more = items.len() > limit;
    items.truncate(limit);
    let next_cursor = more.then(|| items.last().unwrap().id.clone());
    Ok(Page { items, next_cursor })
}

pub fn issued_invitations(
    connection: &Connection,
    principal: &Principal,
    organization: &str,
    query: PageQuery,
) -> Result<Page<IssuedInvitation>, ServiceError> {
    let allowed: bool = connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM memberships WHERE organization_id=?1 AND issuer=?2 AND subject=?3 AND active=1 AND role IN ('owner','admin'))",
        params![organization, principal.issuer, principal.subject],
        |row| row.get(0),
    )?;
    if !allowed {
        return Err(ServiceError::Forbidden);
    }
    let (after, limit) = query.bounds()?;
    let now = now_seconds();
    let mut statement = connection.prepare(
        "SELECT i.id,i.organization_id,o.policy_revision,i.target_issuer,i.target_subject,i.role,i.expires_at,
                CASE WHEN i.status='pending' AND i.expires_at<=?3 THEN 'expired' ELSE i.status END
         FROM invitations i JOIN organizations o ON o.id=i.organization_id
         WHERE i.organization_id=?1 AND i.id>?2 ORDER BY i.id LIMIT ?4",
    )?;
    let rows = statement.query_map(params![organization, after, now, limit + 1], |row| {
        Ok(IssuedInvitation {
            id: row.get(0)?,
            organization_id: row.get(1)?,
            policy_revision: row.get::<_, i64>(2)?.to_string(),
            target_issuer: row.get(3)?,
            target_subject: row.get(4)?,
            role: row.get(5)?,
            expires_at: row.get(6)?,
            status: row.get(7)?,
        })
    })?;
    let mut items = rows.collect::<Result<Vec<_>, _>>()?;
    let more = items.len() > limit;
    items.truncate(limit);
    let next_cursor = more.then(|| items.last().unwrap().id.clone());
    Ok(Page { items, next_cursor })
}
