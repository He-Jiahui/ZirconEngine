use super::*;
use crate::service::storage::Database;

#[path = "invitation_authority_tests.rs"]
mod invitation_authority;

fn principal(subject: &str) -> Principal {
    Principal {
        issuer: "https://identity.example/realm".into(),
        subject: subject.into(),
    }
}
fn request(revision: &str, mutation: Mutation) -> MutationRequest {
    MutationRequest {
        operation_id: uuid::Uuid::new_v4().to_string(),
        expected_policy_revision: revision.into(),
        mutation,
    }
}

fn create(
    connection: &mut Connection,
    principal: &Principal,
    name: &str,
) -> Result<Organization, ServiceError> {
    super::create(
        connection,
        principal,
        &uuid::Uuid::new_v4().to_string(),
        name,
    )
}

fn list(connection: &Connection, principal: &Principal) -> Result<Vec<Organization>, ServiceError> {
    Ok(super::list(connection, principal, PageQuery::default())?.items)
}

#[tokio::test]
async fn tenant_isolation_stale_revision_and_revocation_are_transactional() {
    Database::memory()
        .execute(|connection| {
            let alice = principal("alice");
            let bob = principal("bob");
            let a = create(connection, &alice, "same-name")?;
            let b = create(connection, &bob, "same-name")?;
            assert_eq!(list(connection, &alice)?.len(), 1);
            assert_eq!(list(connection, &bob)?[0].id, b.id);
            assert!(matches!(
                mutate(
                    connection,
                    &bob,
                    &a.id,
                    request(
                        "1",
                        Mutation::CreateProject {
                            name: "cross-tenant".into()
                        }
                    )
                ),
                Err(ServiceError::Forbidden)
            ));
            let invite = mutate(
                connection,
                &alice,
                &a.id,
                request(
                    "1",
                    Mutation::Invite {
                        issuer: bob.issuer.clone(),
                        subject: bob.subject.clone(),
                        role: "member".into(),
                        expires_at: now_seconds() + 60,
                    },
                ),
            )?;
            let accepted = mutate(
                connection,
                &bob,
                &a.id,
                request(
                    "2",
                    Mutation::AcceptInvite {
                        invitation_id: invite.resource_id.clone(),
                    },
                ),
            )?;
            assert_eq!(accepted.policy_revision, "3");
            assert!(matches!(
                mutate(
                    connection,
                    &bob,
                    &a.id,
                    request(
                        "3",
                        Mutation::AcceptInvite {
                            invitation_id: invite.resource_id
                        }
                    )
                ),
                Err(ServiceError::Forbidden)
            ));
            let projects_before: i64 =
                connection.query_row("SELECT count(*) FROM projects", [], |row| row.get(0))?;
            let audit_before: i64 =
                connection.query_row("SELECT count(*) FROM audit_events", [], |row| row.get(0))?;
            assert!(matches!(
                mutate(
                    connection,
                    &alice,
                    &a.id,
                    request(
                        "2",
                        Mutation::CreateProject {
                            name: "stale".into()
                        }
                    )
                ),
                Err(ServiceError::Conflict)
            ));
            assert_eq!(
                connection.query_row("SELECT count(*) FROM projects", [], |row| row
                    .get::<_, i64>(0))?,
                projects_before
            );
            assert_eq!(
                connection.query_row("SELECT count(*) FROM audit_events", [], |row| row
                    .get::<_, i64>(0))?,
                audit_before
            );
            mutate(
                connection,
                &alice,
                &a.id,
                request(
                    "3",
                    Mutation::SetMember {
                        issuer: bob.issuer.clone(),
                        subject: bob.subject.clone(),
                        role: "viewer".into(),
                        active: false,
                    },
                ),
            )?;
            assert!(matches!(
                mutate(
                    connection,
                    &bob,
                    &a.id,
                    request(
                        "4",
                        Mutation::CreateProject {
                            name: "revoked".into()
                        }
                    )
                ),
                Err(ServiceError::Forbidden)
            ));
            assert_eq!(list(connection, &bob)?.len(), 1);
            assert!(connection.execute("DELETE FROM audit_events", []).is_err());
            Ok(())
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn failed_audit_rolls_back_the_entire_mutation() {
    Database::memory().execute(|connection| {
        let alice = principal("alice");
        let organization = create(connection, &alice, "organization")?;
        connection.execute_batch("CREATE TRIGGER fault BEFORE INSERT ON audit_events BEGIN SELECT RAISE(ABORT,'fault'); END;")?;
        assert!(mutate(connection, &alice, &organization.id, request("1", Mutation::CreateProject { name: "rollback".into() })).is_err());
        assert_eq!(list(connection, &alice)?[0].policy_revision, "1");
        assert_eq!(connection.query_row("SELECT count(*) FROM projects", [], |row| row.get::<_, i64>(0))?, 0);
        Ok(())
    }).await.unwrap();
}

#[tokio::test]
async fn cancelled_response_can_be_reconciled_and_retried_without_a_second_commit() {
    let database = Database::memory();
    let worker_database = database.clone();
    let operation_id = uuid::Uuid::new_v4().to_string();
    let worker_id = operation_id.clone();
    let (entered_tx, entered_rx) = tokio::sync::oneshot::channel();
    let (release_tx, release_rx) = std::sync::mpsc::channel();
    let request = tokio::spawn(async move {
        worker_database
            .execute(move |connection| {
                entered_tx.send(()).unwrap();
                release_rx.recv().unwrap();
                super::create(connection, &principal("alice"), &worker_id, "once")
            })
            .await
    });
    entered_rx.await.unwrap();
    request.abort();
    release_tx.send(()).unwrap();
    database
        .execute(move |connection| {
            let alice = principal("alice");
            let replayed = super::create(connection, &alice, &operation_id, "once")?;
            assert_eq!(
                connection.query_row("SELECT count(*) FROM organizations", [], |row| row
                    .get::<_, i64>(0))?,
                1
            );
            assert_eq!(
                connection.query_row("SELECT count(*) FROM audit_events", [], |row| row
                    .get::<_, i64>(0))?,
                1
            );
            assert!(matches!(
                receipt::lookup(connection, &alice, &operation_id)?,
                receipt::OperationStatus::Committed { .. }
            ));
            assert!(matches!(
                receipt::lookup(connection, &principal("bob"), &operation_id)?,
                receipt::OperationStatus::Unknown
            ));
            assert!(matches!(
                super::create(connection, &alice, &operation_id, "different"),
                Err(ServiceError::OperationConflict)
            ));
            assert_eq!(list(connection, &alice)?[0].id, replayed.id);
            Ok(())
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn invitee_can_discover_revision_accept_and_replay_a_receipt() {
    Database::memory()
        .execute(|connection| {
            let alice = principal("alice");
            let bob = principal("bob");
            let organization = create(connection, &alice, "team")?;
            mutate(
                connection,
                &alice,
                &organization.id,
                request(
                    "1",
                    Mutation::Invite {
                        issuer: bob.issuer.clone(),
                        subject: bob.subject.clone(),
                        role: "member".into(),
                        expires_at: now_seconds() + 60,
                    },
                ),
            )?;
            let page = invitations(connection, &bob, PageQuery::default())?;
            assert_eq!(page.items.len(), 1);
            assert!(
                invitations(connection, &principal("outsider"), PageQuery::default())?
                    .items
                    .is_empty()
            );
            let invitation = &page.items[0];
            let acceptance = request(
                &invitation.policy_revision,
                Mutation::AcceptInvite {
                    invitation_id: invitation.id.clone(),
                },
            );
            let accepted = mutate(
                connection,
                &bob,
                &invitation.organization_id,
                acceptance.clone(),
            )?;
            let replayed = mutate(connection, &bob, &invitation.organization_id, acceptance)?;
            assert_eq!(accepted.audit_sequence, replayed.audit_sequence);
            assert_eq!(
                invitations(connection, &bob, PageQuery::default())?.items[0].status,
                "accepted"
            );
            Ok(())
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn issued_invitation_listing_is_paginated_tenant_scoped_and_freshly_authorized() {
    Database::memory()
        .execute(|connection| {
            let owner = principal("owner");
            let admin = principal("admin");
            let member = principal("member");
            let outsider = principal("outsider");
            let organization = create(connection, &owner, "team")?;
            let other = create(connection, &outsider, "other")?;

            connection.execute(
                "INSERT INTO memberships VALUES (?1,?2,?3,'admin',1)",
                params![organization.id, admin.issuer, admin.subject],
            )?;
            connection.execute(
                "INSERT INTO memberships VALUES (?1,?2,?3,'member',1)",
                params![organization.id, member.issuer, member.subject],
            )?;

            let mut issued = Vec::new();
            for (revision, subject) in [("1", "target-a"), ("2", "target-b"), ("3", "target-c")] {
                issued.push(mutate(
                    connection,
                    &admin,
                    &organization.id,
                    request(
                        revision,
                        Mutation::Invite {
                            issuer: "https://partner.example/realm".into(),
                            subject: subject.into(),
                            role: "member".into(),
                            expires_at: now_seconds() + 300,
                        },
                    ),
                )?);
            }
            connection.execute(
                "UPDATE invitations SET expires_at=?1 WHERE id=?2",
                params![now_seconds().saturating_sub(1), issued[2].resource_id],
            )?;

            let first = issued_invitations(
                connection,
                &admin,
                &organization.id,
                PageQuery {
                    after: None,
                    limit: Some(2),
                },
            )?;
            assert_eq!(first.items.len(), 2);
            assert!(first.next_cursor.is_some());
            let second = issued_invitations(
                connection,
                &admin,
                &organization.id,
                PageQuery {
                    after: first.next_cursor.clone(),
                    limit: Some(2),
                },
            )?;
            assert_eq!(second.items.len(), 1);
            assert!(second.next_cursor.is_none());
            let mut listed = first.items;
            listed.extend(second.items);
            assert_eq!(listed.len(), 3);
            assert!(listed.iter().all(|invitation| {
                invitation.organization_id == organization.id
                    && invitation.policy_revision == "4"
                    && invitation.target_issuer == "https://partner.example/realm"
                    && invitation.role == "member"
            }));
            assert_eq!(
                issued_invitations(connection, &owner, &organization.id, PageQuery::default())?
                    .items
                    .len(),
                3
            );
            assert_eq!(
                listed
                    .iter()
                    .find(|invitation| invitation.id == issued[2].resource_id)
                    .unwrap()
                    .status,
                "expired"
            );
            assert!(matches!(
                issued_invitations(connection, &member, &organization.id, PageQuery::default()),
                Err(ServiceError::Forbidden)
            ));
            assert!(matches!(
                issued_invitations(
                    connection,
                    &outsider,
                    &organization.id,
                    PageQuery::default()
                ),
                Err(ServiceError::Forbidden)
            ));
            assert!(matches!(
                issued_invitations(connection, &owner, &other.id, PageQuery::default()),
                Err(ServiceError::Forbidden)
            ));
            assert!(matches!(
                issued_invitations(
                    connection,
                    &owner,
                    &organization.id,
                    PageQuery {
                        after: Some("not-an-invitation-id".into()),
                        limit: None,
                    }
                ),
                Err(ServiceError::InvalidRequest)
            ));
            assert!(matches!(
                issued_invitations(
                    connection,
                    &owner,
                    &organization.id,
                    PageQuery {
                        after: None,
                        limit: Some(101),
                    }
                ),
                Err(ServiceError::InvalidRequest)
            ));

            let revoked = issued[0].resource_id.clone();
            mutate(
                connection,
                &admin,
                &organization.id,
                request(
                    "4",
                    Mutation::RevokeInvite {
                        invitation_id: revoked.clone(),
                    },
                ),
            )?;
            let after_revoke =
                issued_invitations(connection, &admin, &organization.id, PageQuery::default())?;
            assert_eq!(
                after_revoke
                    .items
                    .iter()
                    .find(|invitation| invitation.id == revoked)
                    .unwrap()
                    .status,
                "revoked"
            );
            assert!(after_revoke
                .items
                .iter()
                .all(|invitation| invitation.policy_revision == "5"));

            mutate(
                connection,
                &owner,
                &organization.id,
                request(
                    "5",
                    Mutation::SetMember {
                        issuer: admin.issuer.clone(),
                        subject: admin.subject.clone(),
                        role: "admin".into(),
                        active: false,
                    },
                ),
            )?;
            assert!(matches!(
                issued_invitations(connection, &admin, &organization.id, PageQuery::default()),
                Err(ServiceError::Forbidden)
            ));
            Ok(())
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn pagination_exposes_every_organization_without_a_silent_cutoff() {
    Database::memory()
        .execute(|connection| {
            let alice = principal("alice");
            for _ in 0..103 {
                create(connection, &alice, "team")?;
            }
            let first = super::list(
                connection,
                &alice,
                PageQuery {
                    after: None,
                    limit: Some(100),
                },
            )?;
            assert_eq!(first.items.len(), 100);
            assert!(first.next_cursor.is_some());
            let second = super::list(
                connection,
                &alice,
                PageQuery {
                    after: first.next_cursor,
                    limit: Some(100),
                },
            )?;
            assert_eq!(second.items.len(), 3);
            assert!(second.next_cursor.is_none());
            assert!(first.items.last().unwrap().id < second.items[0].id);
            Ok(())
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn old_and_new_invitations_cannot_override_owner_membership_changes() {
    for active in [false, true] {
        Database::memory()
            .execute(move |connection| {
                let owner = principal("owner");
                let member = principal("member");
                let organization = create(connection, &owner, "team")?;
                let invite = |role: &str| Mutation::Invite {
                    issuer: member.issuer.clone(),
                    subject: member.subject.clone(),
                    role: role.into(),
                    expires_at: now_seconds() + 60,
                };
                let initial = mutate(
                    connection,
                    &owner,
                    &organization.id,
                    request("1", invite("member")),
                )?;
                let old_admin = mutate(
                    connection,
                    &owner,
                    &organization.id,
                    request("2", invite("admin")),
                )?;
                mutate(
                    connection,
                    &member,
                    &organization.id,
                    request(
                        "3",
                        Mutation::AcceptInvite {
                            invitation_id: initial.resource_id,
                        },
                    ),
                )?;
                mutate(
                    connection,
                    &owner,
                    &organization.id,
                    request(
                        "4",
                        Mutation::SetMember {
                            issuer: member.issuer.clone(),
                            subject: member.subject.clone(),
                            role: "viewer".into(),
                            active,
                        },
                    ),
                )?;
                assert!(matches!(
                    mutate(
                        connection,
                        &member,
                        &organization.id,
                        request(
                            "5",
                            Mutation::AcceptInvite {
                                invitation_id: old_admin.resource_id
                            }
                        )
                    ),
                    Err(ServiceError::Forbidden)
                ));
                assert!(matches!(
                    mutate(
                        connection,
                        &owner,
                        &organization.id,
                        request("5", invite("admin"))
                    ),
                    Err(ServiceError::InvalidRequest)
                ));
                let admin = principal("admin");
                let invitation = mutate(
                    connection,
                    &owner,
                    &organization.id,
                    request(
                        "5",
                        Mutation::Invite {
                            issuer: admin.issuer.clone(),
                            subject: admin.subject.clone(),
                            role: "admin".into(),
                            expires_at: now_seconds() + 60,
                        },
                    ),
                )?;
                mutate(
                    connection,
                    &admin,
                    &organization.id,
                    request(
                        "6",
                        Mutation::AcceptInvite {
                            invitation_id: invitation.resource_id,
                        },
                    ),
                )?;
                assert!(matches!(
                    mutate(
                        connection,
                        &admin,
                        &organization.id,
                        request("7", invite("admin"))
                    ),
                    Err(ServiceError::InvalidRequest)
                ));
                let actual: (String, bool) = connection.query_row(
                    "SELECT role,active FROM memberships WHERE organization_id=?1 AND subject=?2",
                    params![organization.id, member.subject],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )?;
                assert_eq!(actual, ("viewer".into(), active));
                Ok(())
            })
            .await
            .unwrap();
    }
}

#[tokio::test]
async fn ownership_transfer_is_atomic_and_only_current_owner_can_start_it() {
    Database::memory()
        .execute(|connection| {
            let owner = principal("owner");
            let member = principal("member");
            let outsider = principal("outsider");
            let organization = create(connection, &owner, "team")?;
            let invite = mutate(
                connection,
                &owner,
                &organization.id,
                request(
                    "1",
                    Mutation::Invite {
                        issuer: member.issuer.clone(),
                        subject: member.subject.clone(),
                        role: "member".into(),
                        expires_at: now_seconds() + 60,
                    },
                ),
            )?;
            mutate(
                connection,
                &member,
                &organization.id,
                request(
                    "2",
                    Mutation::AcceptInvite {
                        invitation_id: invite.resource_id,
                    },
                ),
            )?;
            assert!(matches!(
                mutate(
                    connection,
                    &outsider,
                    &organization.id,
                    request(
                        "3",
                        Mutation::TransferOwnership {
                            issuer: member.issuer.clone(),
                            subject: member.subject.clone()
                        }
                    )
                ),
                Err(ServiceError::Forbidden)
            ));
            let receipt = mutate(
                connection,
                &owner,
                &organization.id,
                request(
                    "3",
                    Mutation::TransferOwnership {
                        issuer: member.issuer.clone(),
                        subject: member.subject.clone(),
                    },
                ),
            )?;
            assert_eq!(receipt.policy_revision, "4");
            assert_eq!(
                connection.query_row(
                    "SELECT role FROM memberships WHERE organization_id=?1 AND subject='owner'",
                    [organization.id.clone()],
                    |row| row.get::<_, String>(0)
                )?,
                "admin"
            );
            assert_eq!(
                connection.query_row(
                    "SELECT role FROM memberships WHERE organization_id=?1 AND subject='member'",
                    [organization.id],
                    |row| row.get::<_, String>(0)
                )?,
                "owner"
            );
            Ok(())
        })
        .await
        .unwrap();
}
