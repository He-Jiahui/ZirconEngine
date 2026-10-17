use super::*;
use axum::{http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use jsonwebtoken::{encode, EncodingKey, Header};
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    RwLock,
};

fn fixture_keys() -> Vec<serde_json::Value> {
    // These generated keys are exclusively test fixtures, never a deployed identity authority.
    serde_json::from_str(include_str!("../test_keys.json")).unwrap()
}

fn token(key: usize, issuer: &str, audience: &str) -> String {
    let fixtures = fixture_keys();
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(format!("test-key-{key}"));
    let der: Vec<u8> = serde_json::from_value(fixtures[key]["privateKeyDer"].clone()).unwrap();
    encode(&header, &serde_json::json!({"sub":"alice", "iss":issuer, "aud":audience, "exp":now_seconds()+300, "typ":"Bearer"}), &EncodingKey::from_rsa_der(&der)).unwrap()
}

#[test]
fn signed_tokens_require_issuer_audience_and_matching_key() {
    let fixtures = fixture_keys();
    let keys: JwkSet =
        serde_json::from_value(serde_json::json!({"keys":[fixtures[0]["jwk"]]})).unwrap();
    assert!(verify_signature(&token(0, "issuer", "audience"), &keys, "issuer", "audience").is_ok());
    assert!(verify_signature(
        &token(0, "different", "audience"),
        &keys,
        "issuer",
        "audience"
    )
    .is_err());
    assert!(verify_signature(
        &token(0, "issuer", "different"),
        &keys,
        "issuer",
        "audience"
    )
    .is_err());
    assert!(
        verify_signature(&token(1, "issuer", "audience"), &keys, "issuer", "audience").is_err()
    );
    assert!(verify_signature("not-a-jwt", &keys, "issuer", "audience").is_err());
    assert!(verify_signature(&"x".repeat(16385), &keys, "issuer", "audience").is_err());
}

#[tokio::test]
async fn rotated_keys_refresh_once_and_replace_revoked_keys() {
    let fixtures = fixture_keys();
    let document = Arc::new(RwLock::new(
        serde_json::json!({"keys":[fixtures[1]["jwk"]]}),
    ));
    let calls = Arc::new(AtomicUsize::new(0));
    let failed = Arc::new(AtomicBool::new(false));
    let router = Router::new().route(
        "/keys",
        get({
            let document = document.clone();
            let calls = calls.clone();
            let failed = failed.clone();
            move || {
                let document = document.clone();
                let calls = calls.clone();
                let failed = failed.clone();
                async move {
                    calls.fetch_add(1, Ordering::SeqCst);
                    if failed.load(Ordering::SeqCst) {
                        return StatusCode::SERVICE_UNAVAILABLE.into_response();
                    }
                    Json(document.read().unwrap().clone()).into_response()
                }
            }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    let issuer = format!("http://{address}/realm");
    let verifier = OidcVerifier {
        config: ServiceConfig {
            bind: address,
            database: std::env::temp_dir().join("unused.db"),
            issuer: issuer.clone(),
            audience: "hub".into(),
            introspection_client_id: "hub".into(),
            introspection_secret_file: std::env::temp_dir().join("unused.secret"),
            allow_loopback_http: true,
            cloud: crate::service::cloud::CloudConfig {
                root: std::env::temp_dir().join("unused-cloud"),
                key_file: std::env::temp_dir().join("unused-cloud.key"),
            },
            catalog_policy_file: None,
        },
        http: reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(2))
            .build()
            .unwrap(),
        keys: Arc::new(Mutex::new(KeyCache {
            keys: serde_json::from_value(serde_json::json!({"keys":[fixtures[0]["jwk"]]})).unwrap(),
            loaded_at: Instant::now(),
            attempted_at: None,
            refresh_failed: false,
        })),
        jwks_endpoint: format!("http://{address}/keys"),
        introspection_endpoint: String::new(),
        secret: Arc::new(String::new()),
    };
    let old = token(0, &issuer, "hub");
    let rotated = token(1, &issuer, "hub");
    assert!(verifier.verify_cached_signature(&old).await.is_ok());
    let mut tasks = Vec::new();
    for _ in 0..8 {
        let verifier = verifier.clone();
        let rotated = rotated.clone();
        tasks.push(tokio::spawn(async move {
            verifier.verify_cached_signature(&rotated).await
        }));
    }
    for task in tasks {
        assert!(task.await.unwrap().is_ok());
    }
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert!(verifier.verify_cached_signature(&old).await.is_err());
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert!(verifier
        .verify_cached_signature(&token(1, &issuer, "other"))
        .await
        .is_err());

    // Expiry refresh replaces the cache even for a known kid; failures never retain trust.
    *document.write().unwrap() = serde_json::json!({"keys":[fixtures[0]["jwk"]]});
    {
        let mut cache = verifier.keys.lock().await;
        cache.loaded_at = Instant::now() - KEY_TTL;
        cache.attempted_at = None;
    }
    assert!(verifier.verify_cached_signature(&rotated).await.is_err());
    assert!(verifier.verify_cached_signature(&old).await.is_ok());
    failed.store(true, Ordering::SeqCst);
    {
        let mut cache = verifier.keys.lock().await;
        cache.loaded_at = Instant::now() - KEY_TTL;
        cache.attempted_at = None;
    }
    assert!(matches!(
        verifier.verify_cached_signature(&old).await,
        Err(ServiceError::IdentityUnavailable)
    ));
    assert!(matches!(
        verifier.verify_cached_signature(&old).await,
        Err(ServiceError::IdentityUnavailable)
    ));
    assert_eq!(calls.load(Ordering::SeqCst), 3);
    server.abort();
}
