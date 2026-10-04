use super::*;
use crate::service::{organization, storage::Database};
use jsonwebtoken::{encode, EncodingKey, Header};

fn principal(subject: &str) -> Principal {
    Principal {
        issuer: "https://identity.example/realm".into(),
        subject: subject.into(),
    }
}

fn fixture() -> (CatalogPolicy, EncodingKey) {
    let keys: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../../identity/test_keys.json")).unwrap();
    let policy = serde_json::from_value(serde_json::json!({"issuer":"https://catalog.example", "audience":"zircon-hub-catalog", "keys":{"keys":[keys[0]["jwk"]]}, "publishers":{"test-key-0":{"issuer":"https://identity.example/realm","subject":"publisher"}}})).unwrap();
    let der: Vec<u8> = serde_json::from_value(keys[0]["privateKeyDer"].clone()).unwrap();
    (policy, EncodingKey::from_rsa_der(&der))
}

fn release() -> Release {
    Release {
        iss: "https://catalog.example".into(),
        aud: "zircon-hub-catalog".into(),
        sub: "publisher".into(),
        exp: now_seconds() + 600,
        package_id: uuid::Uuid::new_v4().to_string(),
        revision: 1,
        version: "1.0.0".into(),
        name: "Local Plugin".into(),
        kind: "plugin".into(),
        description: "Verified package metadata".into(),
        license_id: "MIT".into(),
        license_text: "MIT test license".into(),
        artifact_digest: "a".repeat(64),
        artifact_size: 1024,
    }
}

fn signed(release: &Release, key: &EncodingKey) -> String {
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some("test-key-0".into());
    encode(&header, release, key).unwrap()
}

fn publication(release: &Release, key: &EncodingKey) -> PublishRequest {
    PublishRequest {
        operation_id: uuid::Uuid::new_v4().to_string(),
        envelope: signed(release, key),
    }
}

#[test]
fn catalog_requires_live_signature_authority_and_lossless_revision() {
    let (mut policy, key) = fixture();
    let mut release = release();
    release.revision = 9_007_199_254_740_993;
    let envelope = signed(&release, &key);
    assert_eq!(
        policy.verify(&envelope).unwrap().1.revision,
        release.revision
    );
    for field in ["iss", "aud", "sub"] {
        let mut changed = serde_json::to_value(&release).unwrap();
        changed[field] = serde_json::json!("wrong");
        let changed: Release = serde_json::from_value(changed).unwrap();
        assert!(policy.verify(&signed(&changed, &key)).is_err());
    }
    let mut expired = release.clone();
    expired.exp = now_seconds() - 1;
    assert!(policy.verify(&signed(&expired, &key)).is_err());
    let mut bytes = envelope.into_bytes();
    let last = bytes.len() - 4;
    bytes[last] = if bytes[last] == b'a' { b'b' } else { b'a' };
    assert!(policy.verify(std::str::from_utf8(&bytes).unwrap()).is_err());
    policy.keys.keys.clear();
    assert!(policy.verify(&signed(&release, &key)).is_err());
}

#[tokio::test]
async fn signed_rollback_and_publisher_impersonation_make_no_catalog_write() {
    Database::memory()
        .execute(|connection| {
            let (policy, key) = fixture();
            let mut release = release();
            release.revision = 2;
            publish(
                connection,
                &principal("publisher"),
                &policy,
                publication(&release, &key),
            )?;
            release.revision = 1;
            assert!(matches!(
                publish(
                    connection,
                    &principal("publisher"),
                    &policy,
                    publication(&release, &key)
                ),
                Err(ServiceError::Conflict)
            ));
            release.revision = 3;
            assert!(matches!(
                publish(
                    connection,
                    &principal("other"),
                    &policy,
                    publication(&release, &key)
                ),
                Err(ServiceError::Forbidden)
            ));
            assert_eq!(
                connection.query_row("SELECT count(*) FROM catalog_releases", [], |row| row
                    .get::<_, i64>(0))?,
                1
            );
            assert_eq!(
                connection.query_row("SELECT count(*) FROM catalog_audit", [], |row| row
                    .get::<_, i64>(0))?,
                1
            );
            Ok(())
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn free_license_acceptance_is_qualified_idempotent_and_audited() {
    Database::memory()
        .execute(|connection| {
            let (policy, key) = fixture();
            let release = release();
            publish(
                connection,
                &principal("publisher"),
                &policy,
                publication(&release, &key),
            )?;
            let alice = principal("alice");
            let bob = principal("bob");
            let organization = organization::create(
                connection,
                &alice,
                &uuid::Uuid::new_v4().to_string(),
                "team",
            )?;
            let operation_id = uuid::Uuid::new_v4().to_string();
            let request = || AcceptLicense {
                operation_id: operation_id.clone(),
                expected_policy_revision: "1".into(),
                package_id: release.package_id.clone(),
                revision: "1".into(),
                license_id: "MIT".into(),
            };
            assert!(matches!(
                accept_license(connection, &bob, &policy, &organization.id, request()),
                Err(ServiceError::Forbidden)
            ));
            accept_license(connection, &alice, &policy, &organization.id, request())?;
            accept_license(connection, &alice, &policy, &organization.id, request())?;
            assert_eq!(
                connection.query_row("SELECT count(*) FROM entitlements", [], |row| row
                    .get::<_, i64>(0))?,
                1
            );
            assert_eq!(
                connection.query_row(
                    "SELECT count(*) FROM audit_events WHERE action='license.accept'",
                    [],
                    |row| row.get::<_, i64>(0)
                )?,
                1
            );
            Ok(())
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn license_acceptance_rejects_noncanonical_revision_alias() {
    Database::memory()
        .execute(|connection| {
            let (policy, key) = fixture();
            let release = release();
            publish(
                connection,
                &principal("publisher"),
                &policy,
                publication(&release, &key),
            )?;
            let alice = principal("alice");
            let organization = organization::create(
                connection,
                &alice,
                &uuid::Uuid::new_v4().to_string(),
                "team",
            )?;
            let result = accept_license(
                connection,
                &alice,
                &policy,
                &organization.id,
                AcceptLicense {
                    operation_id: uuid::Uuid::new_v4().to_string(),
                    expected_policy_revision: "1".into(),
                    package_id: release.package_id,
                    revision: "01".into(),
                    license_id: "MIT".into(),
                },
            );
            assert!(matches!(result, Err(ServiceError::InvalidRequest)));
            assert_eq!(
                connection.query_row("SELECT count(*) FROM entitlements", [], |row| row
                    .get::<_, i64>(0))?,
                0
            );
            Ok(())
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn listing_has_a_cursor_beyond_one_hundred_and_revocation_hides_releases() {
    Database::memory()
        .execute(|connection| {
            let (mut policy, key) = fixture();
            for _ in 0..103 {
                publish(
                    connection,
                    &principal("publisher"),
                    &policy,
                    publication(&release(), &key),
                )?;
            }
            let first = list(
                connection,
                &policy,
                CatalogQuery {
                    after: None,
                    query: None,
                    limit: Some(100),
                },
            )?;
            assert_eq!(first.items.len(), 100);
            assert!(first.next_cursor.is_some());
            let second = list(
                connection,
                &policy,
                CatalogQuery {
                    after: first.next_cursor,
                    query: None,
                    limit: Some(100),
                },
            )?;
            assert_eq!(second.items.len(), 3);
            assert!(second.next_cursor.is_none());
            policy.keys.keys.clear();
            assert!(list(
                connection,
                &policy,
                CatalogQuery {
                    after: None,
                    query: None,
                    limit: Some(100)
                }
            )?
            .items
            .is_empty());
            Ok(())
        })
        .await
        .unwrap();
}
