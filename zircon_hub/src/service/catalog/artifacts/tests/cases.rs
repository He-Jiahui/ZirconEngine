use super::*;
use crate::service::{catalog, organization, storage::Database};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};

fn principal(subject: &str) -> Principal {
    Principal {
        issuer: "https://identity.example/realm".into(),
        subject: subject.into(),
    }
}

fn fixture() -> (CatalogPolicy, catalog::Release, EncodingKey, Vec<u8>) {
    let keys: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../../../identity/test_keys.json")).unwrap();
    let policy = serde_json::from_value(serde_json::json!({
        "issuer": "catalog-test", "audience": "hub-test",
        "keys": {"keys": [keys[0]["jwk"]]},
        "publishers": {"test-key-0": {"issuer": principal("publisher").issuer, "subject": "publisher"}}
    })).unwrap();
    let bytes = b"signed package bytes".to_vec();
    let release = catalog::Release {
        iss: "catalog-test".into(),
        aud: "hub-test".into(),
        sub: "publisher".into(),
        exp: crate::service::identity::now_seconds() + 600,
        package_id: uuid::Uuid::new_v4().to_string(),
        revision: 1,
        version: "1.0.0".into(),
        name: "Package".into(),
        kind: "asset".into(),
        description: String::new(),
        license_id: "test".into(),
        license_text: "Test license".into(),
        artifact_digest: format!("{:x}", Sha256::digest(&bytes)),
        artifact_size: bytes.len() as u64,
    };
    let der: Vec<u8> = serde_json::from_value(keys[0]["privateKeyDer"].clone()).unwrap();
    (policy, release, EncodingKey::from_rsa_der(&der), bytes)
}

fn publish(
    connection: &mut Connection,
    policy: &CatalogPolicy,
    release: &catalog::Release,
    key: &EncodingKey,
) {
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some("test-key-0".into());
    catalog::publish(
        connection,
        &principal("publisher"),
        policy,
        catalog::PublishRequest {
            operation_id: uuid::Uuid::new_v4().to_string(),
            envelope: encode(&header, release, key).unwrap(),
        },
    )
    .unwrap();
}

#[tokio::test]
async fn artifact_upload_requires_publisher_exact_signed_bytes_and_is_idempotent() {
    Database::memory()
        .execute(|connection| {
            let (mut policy, release, key, bytes) = fixture();
            publish(connection, &policy, &release, &key);
            assert!(matches!(
                upload(
                    connection,
                    &principal("other"),
                    &policy,
                    &release.package_id,
                    "1",
                    &bytes
                ),
                Err(ServiceError::Forbidden)
            ));
            assert!(matches!(
                upload(
                    connection,
                    &principal("publisher"),
                    &policy,
                    &release.package_id,
                    "01",
                    &bytes
                ),
                Err(ServiceError::InvalidRequest)
            ));
            let mut altered = bytes.clone();
            altered[0] ^= 1;
            assert!(matches!(
                upload(
                    connection,
                    &principal("publisher"),
                    &policy,
                    &release.package_id,
                    "1",
                    &altered
                ),
                Err(ServiceError::InvalidRequest)
            ));
            assert!(matches!(
                upload(
                    connection,
                    &principal("publisher"),
                    &policy,
                    &release.package_id,
                    "1",
                    &bytes[..1]
                ),
                Err(ServiceError::InvalidRequest)
            ));
            assert_eq!(
                connection.query_row("SELECT count(*) FROM catalog_artifacts", [], |row| row
                    .get::<_, i64>(0))?,
                0
            );
            upload(
                connection,
                &principal("publisher"),
                &policy,
                &release.package_id,
                "1",
                &bytes,
            )?;
            upload(
                connection,
                &principal("publisher"),
                &policy,
                &release.package_id,
                "1",
                &bytes,
            )?;
            assert_eq!(
                connection.query_row("SELECT count(*) FROM catalog_artifacts", [], |row| row
                    .get::<_, i64>(0))?,
                1
            );
            policy.publishers.clear();
            assert!(matches!(
                upload(
                    connection,
                    &principal("publisher"),
                    &policy,
                    &release.package_id,
                    "1",
                    &bytes
                ),
                Err(ServiceError::Forbidden)
            ));
            Ok(())
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn artifact_download_requires_live_membership_license_and_intact_persisted_bytes() {
    Database::memory()
        .execute(|connection| {
            let (mut policy, release, key, bytes) = fixture();
            publish(connection, &policy, &release, &key);
            let alice = principal("alice");
            let organization = organization::create(
                connection,
                &alice,
                &uuid::Uuid::new_v4().to_string(),
                "team",
            )?;
            upload(
                connection,
                &principal("publisher"),
                &policy,
                &release.package_id,
                "1",
                &bytes,
            )?;
            assert!(matches!(
                download(
                    connection,
                    &alice,
                    &policy,
                    &organization.id,
                    &release.package_id,
                    "1"
                ),
                Err(ServiceError::Forbidden)
            ));
            catalog::accept_license(
                connection,
                &alice,
                &policy,
                &organization.id,
                catalog::AcceptLicense {
                    operation_id: uuid::Uuid::new_v4().to_string(),
                    expected_policy_revision: "1".into(),
                    package_id: release.package_id.clone(),
                    revision: "1".into(),
                    license_id: release.license_id.clone(),
                },
            )?;
            assert_eq!(
                download(
                    connection,
                    &alice,
                    &policy,
                    &organization.id,
                    &release.package_id,
                    "1"
                )?,
                bytes
            );
            assert!(matches!(
                download(
                    connection,
                    &principal("other"),
                    &policy,
                    &organization.id,
                    &release.package_id,
                    "1"
                ),
                Err(ServiceError::Forbidden)
            ));
            connection.execute(
                "UPDATE catalog_artifacts SET payload=?1 WHERE digest=?2",
                params![vec![b'x'; bytes.len()], release.artifact_digest],
            )?;
            assert!(matches!(
                download(
                    connection,
                    &alice,
                    &policy,
                    &organization.id,
                    &release.package_id,
                    "1"
                ),
                Err(ServiceError::Storage)
            ));
            policy.publishers.clear();
            assert!(matches!(
                download(
                    connection,
                    &alice,
                    &policy,
                    &organization.id,
                    &release.package_id,
                    "1"
                ),
                Err(ServiceError::Forbidden)
            ));
            Ok(())
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn artifact_capacity_is_explicit_and_missing_bytes_are_not_metadata_success() {
    Database::memory()
        .execute(|connection| {
            let (policy, mut release, key, bytes) = fixture();
            release.artifact_size = MAX_ARTIFACT_BYTES as u64 + 1;
            publish(connection, &policy, &release, &key);
            assert!(matches!(
                upload(
                    connection,
                    &principal("publisher"),
                    &policy,
                    &release.package_id,
                    "1",
                    &bytes
                ),
                Err(ServiceError::Capacity)
            ));
            assert_eq!(
                connection.query_row("SELECT count(*) FROM catalog_artifacts", [], |row| row
                    .get::<_, i64>(0))?,
                0
            );
            assert!(validate_capacity(MAX_CATALOG_BYTES, 1).is_err());
            assert!(validate_capacity(MAX_CATALOG_BYTES - 1, 1).is_ok());
            assert!(validate_capacity(u64::MAX, 1).is_err());
            Ok(())
        })
        .await
        .unwrap();
}
