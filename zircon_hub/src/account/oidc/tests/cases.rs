use super::*;
use axum::{
    extract::Form,
    routing::{get, post},
    Json, Router,
};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

#[test]
fn refresh_transport_errors_preserve_session_and_only_invalid_grant_expires_it() {
    use openidconnect::{core::CoreErrorResponseType, RequestTokenError, StandardErrorResponse};
    assert!(matches!(
        refresh_error(RequestTokenError::Request(
            AccountError::ProviderUnavailable
        )),
        AccountError::ProviderUnavailable
    ));
    for code in [
        CoreErrorResponseType::InvalidClient,
        CoreErrorResponseType::InvalidRequest,
    ] {
        assert!(matches!(
            refresh_error(RequestTokenError::ServerResponse(
                StandardErrorResponse::new(code, None, None)
            )),
            AccountError::ProviderUnavailable
        ));
    }
    assert!(matches!(
        refresh_error(RequestTokenError::ServerResponse(
            StandardErrorResponse::new(CoreErrorResponseType::InvalidGrant, None, None)
        )),
        AccountError::SessionExpired
    ));
}

#[tokio::test]
async fn provider_5xx_cannot_masquerade_as_a_definitive_expired_session() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let router = Router::new().route(
        "/token",
        post(|| async {
            (
                axum::http::StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({"error":"invalid_grant"})),
            )
        }),
    );
    let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    let config = AccountConfig {
        issuer: base.clone(),
        service_url: base.clone(),
        client_id: "desktop".into(),
        callback_port: 8480,
        allow_loopback_http: true,
        operation_journal_path: None,
    };
    let request = openidconnect::http::Request::builder()
        .method("POST")
        .uri(format!("{base}/token"))
        .body(Vec::new())
        .unwrap();
    assert!(matches!(
        provider_request(&config, &http().unwrap(), request).await,
        Err(AccountError::ProviderUnavailable)
    ));
    server.abort();
    let _ = server.await;
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn signed(claims: &serde_json::Value) -> String {
    let keys: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../../../service/identity/test_keys.json")).unwrap();
    let der: Vec<u8> = serde_json::from_value(keys[0]["privateKeyDer"].clone()).unwrap();
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some("test-key-0".into());
    encode(&header, claims, &EncodingKey::from_rsa_der(&der)).unwrap()
}

#[tokio::test]
async fn refresh_verifies_signature_claims_nonce_and_original_subject() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let issuer = format!("{base}/realm");
    let claims = serde_json::json!({"iss":issuer, "sub":"alice", "aud":"desktop", "iat":now(), "exp":now()+300, "nonce":"fixture-nonce", "preferred_username":"Alice"});
    let response_claims = Arc::new(RwLock::new(claims.clone()));
    let metadata = serde_json::json!({"issuer":issuer, "authorization_endpoint":format!("{base}/auth"), "token_endpoint":format!("{base}/token"), "jwks_uri":format!("{base}/keys"), "response_types_supported":["code"], "subject_types_supported":["public"], "id_token_signing_alg_values_supported":["RS256"]});
    let keys: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../../../service/identity/test_keys.json")).unwrap();
    let jwks = serde_json::json!({"keys":[keys[0]["jwk"]]});
    let router = Router::new()
        .route("/realm/.well-known/openid-configuration", get(move || { let metadata = metadata.clone(); async { Json(metadata) } }))
        .route("/keys", get(move || { let jwks = jwks.clone(); async { Json(jwks) } }))
        .route("/token", post({ let response_claims = response_claims.clone(); move |Form(form): Form<HashMap<String, String>>| {
            let response_claims = response_claims.clone();
            async move {
                assert_eq!(form.get("grant_type").map(String::as_str), Some("refresh_token"));
                assert_eq!(form.get("client_id").map(String::as_str), Some("desktop"));
                Json(serde_json::json!({"access_token":"fixture-access", "refresh_token":"fixture-rotated", "token_type":"Bearer", "expires_in":120, "id_token":signed(&response_claims.read().unwrap())}))
            }
        }}));
    let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    let config = AccountConfig {
        issuer: issuer.clone(),
        client_id: "desktop".into(),
        service_url: base,
        callback_port: 8480,
        allow_loopback_http: true,
        operation_journal_path: None,
    };
    let saved = SavedSession {
        issuer,
        subject: "alice".into(),
        nonce: "fixture-nonce".into(),
        refresh_token: "fixture-refresh".into(),
        revoke_only: false,
    };
    let authenticated = authenticate(&config, Some(&saved)).await.unwrap();
    assert_eq!(authenticated.subject, "alice");
    assert_eq!(
        authenticated.tokens.refresh_token().unwrap().secret(),
        "fixture-rotated"
    );
    let mut refreshed_without_nonce = claims.clone();
    refreshed_without_nonce
        .as_object_mut()
        .unwrap()
        .remove("nonce");
    *response_claims.write().unwrap() = refreshed_without_nonce;
    let refreshed = authenticate(&config, Some(&saved)).await.unwrap();
    assert_eq!(refreshed.subject, "alice");
    assert_eq!(refreshed.nonce, saved.nonce);
    for (field, value) in [
        ("iss", serde_json::json!("https://wrong.example")),
        ("sub", serde_json::json!("bob")),
        ("aud", serde_json::json!("other")),
        ("nonce", serde_json::json!("wrong")),
        ("exp", serde_json::json!(now() - 300)),
    ] {
        let mut changed = claims.clone();
        changed[field] = value;
        *response_claims.write().unwrap() = changed;
        assert!(
            matches!(
                authenticate(&config, Some(&saved)).await,
                Err(AccountError::InvalidIdentity)
            ),
            "claim {field} must be checked"
        );
    }
    server.abort();
}

#[tokio::test]
async fn provider_transport_rejects_a_cross_origin_endpoint_before_sending() {
    let config = AccountConfig {
        issuer: "https://identity.example/realm".into(),
        client_id: "desktop".into(),
        service_url: "https://service.example".into(),
        callback_port: 8480,
        allow_loopback_http: false,
        operation_journal_path: None,
    };
    let request = openidconnect::http::Request::builder()
        .uri("http://127.0.0.1:1/private")
        .body(Vec::new())
        .unwrap();
    assert!(matches!(
        provider_request(&config, &http().unwrap(), request).await,
        Err(AccountError::Configuration)
    ));
}
