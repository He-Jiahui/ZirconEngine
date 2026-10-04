use super::*;

fn apply(
    connection: &mut Connection,
    actor: &Principal,
    organization: &str,
    mutation: Mutation,
) -> Result<Receipt, ServiceError> {
    let revision: i64 = connection.query_row(
        "SELECT policy_revision FROM organizations WHERE id=?1",
        [organization],
        |row| row.get(0),
    )?;
    mutate(
        connection,
        actor,
        organization,
        request(&revision.to_string(), mutation),
    )
}

fn invite(target: &Principal, role: &str) -> Mutation {
    Mutation::Invite {
        issuer: target.issuer.clone(),
        subject: target.subject.clone(),
        role: role.into(),
        expires_at: now_seconds() + 600,
    }
}

fn accept(invitation: &Receipt) -> Mutation {
    Mutation::AcceptInvite {
        invitation_id: invitation.resource_id.clone(),
    }
}

fn set_member(member: &Principal, role: &str, active: bool) -> Mutation {
    Mutation::SetMember {
        issuer: member.issuer.clone(),
        subject: member.subject.clone(),
        role: role.into(),
        active,
    }
}

fn setup(
    connection: &mut Connection,
) -> Result<(Principal, Principal, Organization), ServiceError> {
    let owner = principal("owner");
    let admin = principal("admin");
    let organization = create(connection, &owner, "team")?;
    let invitation = apply(
        connection,
        &owner,
        &organization.id,
        invite(&admin, "admin"),
    )?;
    apply(connection, &admin, &organization.id, accept(&invitation))?;
    Ok((owner, admin, organization))
}

#[tokio::test]
async fn inviter_revocation_is_permanent_for_pending_invites_and_audited() {
    for (role, active) in [("admin", false), ("viewer", true), ("member", true)] {
        Database::memory().execute(move |connection| {
            let (owner, admin, organization) = setup(connection)?;
            let target = principal("target");
            let invitation = apply(connection, &admin, &organization.id, invite(&target, "admin"))?;
            apply(connection, &owner, &organization.id, set_member(&admin, role, active))?;
            for restored in [false, true] {
                if restored {
                    apply(connection, &owner, &organization.id, set_member(&admin, "admin", true))?;
                }
                let revision: i64 = connection.query_row("SELECT policy_revision FROM organizations WHERE id=?1", [&organization.id], |row| row.get(0))?;
                let acceptance = request(&revision.to_string(), accept(&invitation));
                for _ in 0..2 {
                    assert!(matches!(mutate(connection, &target, &organization.id, acceptance.clone()), Err(ServiceError::Forbidden)));
                }
                assert!(matches!(receipt::lookup(connection, &target, &acceptance.operation_id)?, receipt::OperationStatus::Unknown));
                assert_eq!(connection.query_row("SELECT count(*) FROM memberships WHERE organization_id=?1 AND subject='target'", [&organization.id], |row| row.get::<_, i64>(0))?, 0);
                assert_eq!(connection.query_row("SELECT policy_revision FROM organizations WHERE id=?1", [&organization.id], |row| row.get::<_, i64>(0))?, revision);
                assert_eq!(connection.query_row("SELECT status FROM invitations WHERE id=?1", [&invitation.resource_id], |row| row.get::<_, String>(0))?, "revoked");
                assert_eq!(connection.query_row("SELECT outcome FROM audit_events WHERE action='invite.accept' ORDER BY sequence DESC LIMIT 1", [], |row| row.get::<_, String>(0))?, "denied");
            }
            Ok(())
        }).await.unwrap();
    }
}

#[tokio::test]
async fn acceptance_checks_fresh_qualified_inviter_membership_in_its_tenant() {
    Database::memory().execute(|connection| {
        let (_, admin, organization) = setup(connection)?;
        let target = principal("target");
        let invitation = apply(connection, &admin, &organization.id, invite(&target, "member"))?;
        // A pending row is not authority, even if the issuer has authority in a different tenant.
        create(connection, &admin, "other tenant")?;
        connection.execute("UPDATE memberships SET active=0 WHERE organization_id=?1 AND issuer=?2 AND subject=?3", params![organization.id, admin.issuer, admin.subject])?;
        connection.execute("INSERT INTO memberships VALUES (?1,'https://other.example/realm',?2,'admin',1)", params![organization.id, admin.subject])?;
        assert!(matches!(apply(connection, &target, &organization.id, accept(&invitation)), Err(ServiceError::Forbidden)));
        assert_eq!(connection.query_row("SELECT status FROM invitations WHERE id=?1", [&invitation.resource_id], |row| row.get::<_, String>(0))?, "pending");
        assert_eq!(connection.query_row("SELECT count(*) FROM memberships WHERE organization_id=?1 AND subject='target'", [&organization.id], |row| row.get::<_, i64>(0))?, 0);
        Ok(())
    }).await.unwrap();
}

#[tokio::test]
async fn acceptance_target_and_organization_are_qualified_before_revision_conflict() {
    Database::memory()
        .execute(|connection| {
            let (owner, admin, organization) = setup(connection)?;
            let target = principal("target");
            let invitation = apply(
                connection,
                &admin,
                &organization.id,
                invite(&target, "member"),
            )?;
            let other = create(connection, &owner, "other")?;
            let wrong_issuer = Principal {
                issuer: "https://other.example/realm".into(),
                subject: target.subject.clone(),
            };
            assert!(matches!(
                mutate(
                    connection,
                    &wrong_issuer,
                    &organization.id,
                    request("1", accept(&invitation))
                ),
                Err(ServiceError::Forbidden)
            ));
            assert!(matches!(
                mutate(
                    connection,
                    &target,
                    &other.id,
                    request("1", accept(&invitation))
                ),
                Err(ServiceError::Forbidden)
            ));
            let audit_before: i64 =
                connection.query_row("SELECT count(*) FROM audit_events", [], |row| row.get(0))?;
            assert!(matches!(
                mutate(
                    connection,
                    &target,
                    &organization.id,
                    request("1", accept(&invitation))
                ),
                Err(ServiceError::Conflict)
            ));
            assert_eq!(
                connection.query_row("SELECT count(*) FROM audit_events", [], |row| row
                    .get::<_, i64>(0))?,
                audit_before
            );
            apply(connection, &target, &organization.id, accept(&invitation))?;
            Ok(())
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn failed_revocation_audit_rolls_back_member_and_issued_invites() {
    Database::memory().execute(|connection| {
        let (owner, admin, organization) = setup(connection)?;
        let target = principal("target");
        let invitation = apply(connection, &admin, &organization.id, invite(&target, "member"))?;
        connection.execute_batch("CREATE TRIGGER audit_fault BEFORE INSERT ON audit_events BEGIN SELECT RAISE(ABORT,'fault'); END;")?;
        assert!(matches!(apply(connection, &owner, &organization.id, set_member(&admin, "viewer", false)), Err(ServiceError::Storage)));
        connection.execute_batch("DROP TRIGGER audit_fault;")?;
        assert_eq!(connection.query_row("SELECT status FROM invitations WHERE id=?1", [&invitation.resource_id], |row| row.get::<_, String>(0))?, "pending");
        assert_eq!(connection.query_row("SELECT active FROM memberships WHERE organization_id=?1 AND subject='admin'", [&organization.id], |row| row.get::<_, bool>(0))?, true);
        apply(connection, &target, &organization.id, accept(&invitation))?;
        Ok(())
    }).await.unwrap();
}

#[tokio::test]
async fn accepted_receipt_retry_checks_authority_without_restoring_revoked_access() {
    for revoke_inviter in [false, true] {
        Database::memory().execute(move |connection| {
            let (owner, admin, organization) = setup(connection)?;
            let target = principal("target");
            let invitation = apply(connection, &admin, &organization.id, invite(&target, "member"))?;
            let acceptance = request(&invitation.policy_revision, accept(&invitation));
            let committed = mutate(connection, &target, &organization.id, acceptance.clone())?;
            let replayed = mutate(connection, &target, &organization.id, acceptance.clone())?;
            assert_eq!(committed.audit_sequence, replayed.audit_sequence);
            let revoked = if revoke_inviter { &admin } else { &target };
            apply(connection, &owner, &organization.id, set_member(revoked, "viewer", false))?;
            assert!(matches!(mutate(connection, &target, &organization.id, acceptance.clone()), Err(ServiceError::Forbidden)));
            assert!(matches!(receipt::lookup(connection, &target, &acceptance.operation_id)?, receipt::OperationStatus::Committed { .. }));
            assert!(!connection.query_row("SELECT active FROM memberships WHERE organization_id=?1 AND issuer=?2 AND subject=?3", params![organization.id, revoked.issuer, revoked.subject], |row| row.get::<_, bool>(0))?);
            assert_eq!(connection.query_row("SELECT count(*) FROM audit_events WHERE action='invite.accept' AND outcome='allowed'", [], |row| row.get::<_, i64>(0))?, 2);
            Ok(())
        }).await.unwrap();
    }
}

#[tokio::test]
async fn reactivated_inviter_cannot_reuse_revoked_grant_but_can_issue_a_fresh_one() {
    Database::memory().execute(|connection| {
        let (owner, admin, organization) = setup(connection)?;
        let target = principal("target");
        let stale = apply(connection, &admin, &organization.id, invite(&target, "admin"))?;

        // Demotion permanently revokes grants issued by the old admin.
        apply(connection, &owner, &organization.id, set_member(&admin, "viewer", true))?;
        apply(connection, &owner, &organization.id, set_member(&admin, "admin", true))?;
        assert_eq!(
            connection.query_row(
                "SELECT status FROM invitations WHERE organization_id=?1 AND id=?2",
                params![organization.id, stale.resource_id],
                |row| row.get::<_, String>(0),
            )?,
            "revoked"
        );
        assert!(matches!(
            apply(connection, &target, &organization.id, accept(&stale)),
            Err(ServiceError::Forbidden)
        ));

        // A newly issued grant is valid after authority is restored.
        let fresh = apply(connection, &admin, &organization.id, invite(&target, "member"))?;
        apply(connection, &target, &organization.id, accept(&fresh))?;
        assert_eq!(
            connection.query_row(
                "SELECT role FROM memberships WHERE organization_id=?1 AND issuer=?2 AND subject=?3",
                params![organization.id, target.issuer, target.subject],
                |row| row.get::<_, String>(0),
            )?,
            "member"
        );
        Ok(())
    }).await.unwrap();
}
