use super::*;
use crate::service::{
    cloud::tests::support::{digest, manifest, seed, Files},
    config::ServiceConfig,
    identity::now_seconds,
    organization::{self, Mutation, MutationRequest},
};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use openidconnect::reqwest;
use std::sync::atomic::{AtomicBool, Ordering};

struct Server(tokio::task::JoinHandle<()>);

impl Drop for Server {
    fn drop(&mut self) {
        self.0.abort();
    }
}

async fn response_json(response: reqwest::Response) -> serde_json::Value {
    serde_json::from_slice(&response.bytes().await.unwrap()).unwrap()
}

#[tokio::test]
async fn http_cloud_routes_require_verified_identity_and_return_replayable_conflict() {
    let files = Files::new();
    let (database, store) = files.database().await;
    let fixtures: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../../../identity/test_keys.json")).unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let issuer = format!("http://{}/realm", listener.local_addr().unwrap());
    let active = Arc::new(AtomicBool::new(true));
    let metadata = serde_json::json!({
        "issuer":issuer, "authorization_endpoint":format!("{issuer}/authorize"),
        "token_endpoint":format!("{issuer}/token"), "jwks_uri":format!("{issuer}/keys"),
        "response_types_supported":["code"], "subject_types_supported":["public"],
        "id_token_signing_alg_values_supported":["RS256"]
    });
    let keys = serde_json::json!({"keys":[fixtures[0]["jwk"]]});
    let identity_app = Router::new()
        .route("/realm/.well-known/openid-configuration", get(move || { let metadata = metadata.clone(); async move { Json(metadata) } }))
        .route("/realm/keys", get(move || { let keys = keys.clone(); async move { Json(keys) } }))
        .route("/realm/protocol/openid-connect/token/introspect", post({
            let active = active.clone();
            move || { let active = active.clone(); async move { Json(serde_json::json!({"active":active.load(Ordering::SeqCst),"sub":"alice","exp":now_seconds()+60})) } }
        }));
    let _identity_server = Server(tokio::spawn(async move {
        axum::serve(listener, identity_app).await.unwrap()
    }));
    let secret = files.path.join("introspection.secret");
    std::fs::write(&secret, "fixture-only").unwrap();
    let identity = OidcVerifier::discover(ServiceConfig {
        bind: "127.0.0.1:0".parse().unwrap(),
        database: files.path.join("unused.db"),
        issuer: issuer.clone(),
        audience: "hub".into(),
        introspection_client_id: "hub".into(),
        introspection_secret_file: secret,
        allow_loopback_http: true,
        cloud: files.config.clone(),
        catalog_policy_file: None,
    })
    .await
    .unwrap();
    let owner = Principal {
        issuer: issuer.clone(),
        subject: "alice".into(),
    };
    let (organization, project) = database
        .execute(move |connection| {
            let (organization, project) = seed(connection, &owner);
            for revision in 1..=3 {
                organization::mutate(
                    connection,
                    &owner,
                    &organization,
                    MutationRequest {
                        operation_id: uuid::Uuid::new_v4().to_string(),
                        expected_policy_revision: revision.to_string(),
                        mutation: Mutation::Invite {
                            issuer: owner.issuer.clone(),
                            subject: format!("target-{revision}"),
                            role: "member".into(),
                            expires_at: now_seconds() + 60,
                        },
                    },
                )?;
            }
            Ok((organization, project))
        })
        .await
        .unwrap();
    let app = super::super::router(identity, database.clone(), None, store);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let cloud = format!("{base}/v1/organizations/{organization}/projects/{project}/cloud");
    let _service = Server(tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap()
    }));
    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap();
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some("test-key-0".into());
    let der: Vec<u8> = serde_json::from_value(fixtures[0]["privateKeyDer"].clone()).unwrap();
    let token = encode(&header, &serde_json::json!({"sub":"alice","iss":issuer,"aud":"hub","exp":now_seconds()+60,"typ":"Bearer"}), &EncodingKey::from_rsa_der(&der)).unwrap();

    let issued_url = format!("{base}/v1/organizations/{organization}/invitations");
    let first = http
        .get(format!("{issued_url}?limit=2"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(first.status(), StatusCode::OK);
    let first = response_json(first).await;
    assert_eq!(first["items"].as_array().unwrap().len(), 2);
    let cursor = first["nextCursor"].as_str().unwrap();
    assert!(first["items"].as_array().unwrap().iter().all(|invitation| {
        invitation["organizationId"] == organization
            && invitation["policyRevision"] == "4"
            && invitation["targetIssuer"] == issuer
            && invitation["role"] == "member"
            && invitation["status"] == "pending"
            && invitation["expiresAt"].as_u64().is_some()
    }));
    let second = http
        .get(format!("{issued_url}?limit=2&after={cursor}"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(second.status(), StatusCode::OK);
    let second = response_json(second).await;
    assert_eq!(second["items"].as_array().unwrap().len(), 1);
    assert!(second["nextCursor"].is_null());

    let revoked = first["items"][0]["id"].as_str().unwrap().to_owned();
    let revoke = http
        .post(format!("{base}/v1/organizations/{organization}/mutations"))
        .bearer_auth(&token)
        .header(header::CONTENT_TYPE, "application/json")
        .body(
            serde_json::to_vec(&serde_json::json!({
                "operationId": uuid::Uuid::new_v4().to_string(),
                "expectedPolicyRevision": "4",
                "mutation": {"action": "revoke-invite", "invitation_id": revoked}
            }))
            .unwrap(),
        )
        .send()
        .await
        .unwrap();
    assert_eq!(revoke.status(), StatusCode::OK);
    let after_revoke = response_json(
        http.get(&issued_url)
            .bearer_auth(&token)
            .send()
            .await
            .unwrap(),
    )
    .await;
    let revoked_invitation = after_revoke["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|invitation| invitation["id"] == revoked)
        .unwrap();
    assert_eq!(revoked_invitation["status"], "revoked");
    assert_eq!(revoked_invitation["policyRevision"], "5");

    assert_eq!(
        http.get(format!("{cloud}/head"))
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        http.get(format!("{cloud}/head"))
            .bearer_auth(&token)
            .header(header::ORIGIN, "http://untrusted.test")
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        http.get(format!("{cloud}/head"))
            .bearer_auth("not-a-token")
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::UNAUTHORIZED
    );
    let bytes = b"cloud route body";
    let blob_url = format!("{cloud}/blobs/{}", digest(bytes));
    assert_eq!(
        http.put(&blob_url)
            .bearer_auth(&token)
            .body(bytes.to_vec())
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        http.get(&blob_url)
            .bearer_auth(&token)
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::FORBIDDEN
    );
    let commit_url = format!("{cloud}/commit");
    let request = serde_json::json!({"operationId":uuid::Uuid::new_v4().to_string(),"baseRevision":"0","manifest":manifest(bytes)});
    let committed = http
        .post(&commit_url)
        .bearer_auth(&token)
        .header(header::CONTENT_TYPE, "application/json")
        .body(serde_json::to_vec(&request).unwrap())
        .send()
        .await
        .unwrap();
    assert_eq!(committed.status(), StatusCode::OK);
    assert_eq!(response_json(committed).await["snapshot"]["revision"], "1");
    let mut stale = request.clone();
    stale["operationId"] = serde_json::json!(uuid::Uuid::new_v4().to_string());
    let conflict = http
        .post(&commit_url)
        .bearer_auth(&token)
        .header(header::CONTENT_TYPE, "application/json")
        .body(serde_json::to_vec(&stale).unwrap())
        .send()
        .await
        .unwrap();
    assert_eq!(conflict.status(), StatusCode::CONFLICT);
    let conflict = response_json(conflict).await;
    assert_eq!(conflict["status"], "conflict");
    assert_eq!(conflict["currentRevision"], "1");
    let replay = http
        .post(&commit_url)
        .bearer_auth(&token)
        .header(header::CONTENT_TYPE, "application/json")
        .body(serde_json::to_vec(&stale).unwrap())
        .send()
        .await
        .unwrap();
    assert_eq!(replay.status(), StatusCode::CONFLICT);
    assert_eq!(response_json(replay).await, conflict);
    let operation = response_json(
        http.get(format!(
            "{base}/v1/operations/{}",
            stale["operationId"].as_str().unwrap()
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap(),
    )
    .await;
    assert_eq!(operation["result"], conflict);
    let download = http
        .get(&blob_url)
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(download.status(), StatusCode::OK);
    assert_eq!(download.headers()[header::CACHE_CONTROL], "no-store");
    assert_eq!(download.bytes().await.unwrap().as_ref(), bytes);
    let retention_url = format!("{cloud}/retention");
    assert_eq!(
        http.get(&retention_url).send().await.unwrap().status(),
        StatusCode::UNAUTHORIZED
    );
    let initial = http
        .get(&retention_url)
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(initial.status(), StatusCode::OK);
    assert_eq!(initial.headers()[header::CACHE_CONTROL], "no-store");
    let initial = response_json(initial).await;
    assert_eq!(initial["revision"], "0");
    assert!(initial["policy"]["keepLatest"].is_null());
    let retention_request = serde_json::json!({
        "operationId": uuid::Uuid::new_v4().to_string(), "expectedRevision": "0",
        "policy": {"keepLatest": 1, "trashSeconds": 0, "legalHold": false}
    });
    let changed = http
        .post(&retention_url)
        .bearer_auth(&token)
        .header(header::CONTENT_TYPE, "application/json")
        .body(serde_json::to_vec(&retention_request).unwrap())
        .send()
        .await
        .unwrap();
    assert_eq!(changed.status(), StatusCode::OK);
    let changed = response_json(changed).await;
    assert_eq!(changed["status"], "retentionUpdated");
    assert_eq!(changed["retention"]["revision"], "1");
    let replay = http
        .post(&retention_url)
        .bearer_auth(&token)
        .header(header::CONTENT_TYPE, "application/json")
        .body(serde_json::to_vec(&retention_request).unwrap())
        .send()
        .await
        .unwrap();
    assert_eq!(replay.status(), StatusCode::OK);
    assert_eq!(response_json(replay).await, changed);
    let retention_operation = format!(
        "{base}/v1/operations/{}",
        retention_request["operationId"].as_str().unwrap()
    );
    assert_eq!(
        response_json(
            http.get(&retention_operation)
                .bearer_auth(&token)
                .send()
                .await
                .unwrap()
        )
        .await["result"],
        changed
    );
    let mut stale_retention = retention_request.clone();
    stale_retention["operationId"] = serde_json::json!(uuid::Uuid::new_v4().to_string());
    let conflict = http
        .post(&retention_url)
        .bearer_auth(&token)
        .header(header::CONTENT_TYPE, "application/json")
        .body(serde_json::to_vec(&stale_retention).unwrap())
        .send()
        .await
        .unwrap();
    assert_eq!(conflict.status(), StatusCode::CONFLICT);
    assert_eq!(response_json(conflict).await["currentRevision"], "1");
    let maintenance_url = format!("{cloud}/maintenance");
    let maintenance_request = serde_json::json!({"operationId": uuid::Uuid::new_v4().to_string()});
    let maintenance = http
        .post(&maintenance_url)
        .bearer_auth(&token)
        .header(header::CONTENT_TYPE, "application/json")
        .body(serde_json::to_vec(&maintenance_request).unwrap())
        .send()
        .await
        .unwrap();
    assert_eq!(maintenance.status(), StatusCode::OK);
    let maintenance = response_json(maintenance).await;
    assert_eq!(maintenance["status"], "maintenanceCompleted");
    assert_eq!(maintenance["pendingObjects"], 0);
    let replay = http
        .post(&maintenance_url)
        .bearer_auth(&token)
        .header(header::CONTENT_TYPE, "application/json")
        .body(serde_json::to_vec(&maintenance_request).unwrap())
        .send()
        .await
        .unwrap();
    assert_eq!(replay.status(), StatusCode::OK);
    assert_eq!(response_json(replay).await, maintenance);
    let maintenance_operation = format!(
        "{base}/v1/operations/{}",
        maintenance_request["operationId"].as_str().unwrap()
    );
    assert_eq!(
        response_json(
            http.get(&maintenance_operation)
                .bearer_auth(&token)
                .send()
                .await
                .unwrap()
        )
        .await["result"],
        maintenance
    );
    assert_eq!(
        http.put(&blob_url)
            .bearer_auth(&token)
            .body(vec![0; cloud::MAX_BLOB_BYTES + 1])
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::TOO_MANY_REQUESTS
    );
    assert_eq!(
        http.post(format!("{base}/v1/organizations"))
            .bearer_auth(&token)
            .header(header::CONTENT_TYPE, "application/json")
            .body(vec![b' '; 65537])
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::PAYLOAD_TOO_LARGE
    );
    database
        .execute({
            let organization = organization.clone();
            move |connection| {
                connection.execute(
                    "UPDATE memberships SET role='viewer' WHERE organization_id=?1 AND issuer=?2 AND subject=?3",
                    rusqlite::params![organization, issuer, "alice"],
                )?;
                Ok(())
            }
        })
        .await
        .unwrap();
    assert_eq!(
        http.get(&issued_url)
            .bearer_auth(&token)
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        http.get(&retention_url)
            .bearer_auth(&token)
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );
    assert_eq!(
        http.post(&retention_url)
            .bearer_auth(&token)
            .header(header::CONTENT_TYPE, "application/json")
            .body(serde_json::to_vec(&retention_request).unwrap())
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        http.post(&maintenance_url)
            .bearer_auth(&token)
            .header(header::CONTENT_TYPE, "application/json")
            .body(serde_json::to_vec(&maintenance_request).unwrap())
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::FORBIDDEN
    );
    database
        .execute(move |connection| {
            connection.execute(
                "UPDATE memberships SET active=0 WHERE organization_id=?1",
                [&organization],
            )?;
            Ok(())
        })
        .await
        .unwrap();
    assert_eq!(
        http.get(&issued_url)
            .bearer_auth(&token)
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        http.get(&blob_url)
            .bearer_auth(&token)
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        http.get(format!(
            "{base}/v1/operations/{}",
            stale["operationId"].as_str().unwrap()
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap()
        .status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        http.get(&retention_url)
            .bearer_auth(&token)
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        http.get(&retention_operation)
            .bearer_auth(&token)
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        http.get(&maintenance_operation)
            .bearer_auth(&token)
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::FORBIDDEN
    );
    active.store(false, Ordering::SeqCst);
    assert_eq!(
        http.get(format!("{cloud}/head"))
            .bearer_auth(&token)
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::UNAUTHORIZED
    );
}

#[test]
fn response_bytes_hold_transfer_permit_until_every_clone_is_dropped() {
    let slots = Arc::new(Semaphore::new(1));
    let permit = slots.clone().try_acquire_owned().unwrap();
    let bytes = Bytes::from_owner(TransferBytes {
        bytes: vec![1, 2, 3],
        _permit: permit,
    });
    let clone = bytes.clone();
    drop(bytes);
    assert!(slots.try_acquire().is_err());
    drop(clone);
    assert!(slots.try_acquire().is_ok());
}
