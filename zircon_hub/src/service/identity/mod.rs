use super::{config::ServiceConfig, error::ServiceError};
use jsonwebtoken::{decode, decode_header, jwk::JwkSet, Algorithm, DecodingKey, Validation};
use openidconnect::{core::CoreProviderMetadata, reqwest, IssuerUrl};
use serde::Deserialize;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct Principal {
    pub issuer: String,
    pub subject: String,
}

#[derive(Clone)]
pub struct OidcVerifier {
    config: ServiceConfig,
    http: reqwest::Client,
    keys: Arc<Mutex<KeyCache>>,
    jwks_endpoint: String,
    introspection_endpoint: String,
    secret: Arc<String>,
}

struct KeyCache {
    keys: JwkSet,
    loaded_at: Instant,
    attempted_at: Option<Instant>,
    refresh_failed: bool,
}

const KEY_TTL: Duration = Duration::from_secs(60);
const REFRESH_BACKOFF: Duration = Duration::from_secs(5);

#[derive(Clone, Deserialize)]
struct Claims {
    sub: String,
    iss: String,
    exp: u64,
    #[serde(default)]
    typ: String,
}

#[derive(Deserialize)]
struct Introspection {
    active: bool,
    sub: Option<String>,
    exp: Option<u64>,
}

impl OidcVerifier {
    pub async fn discover(config: ServiceConfig) -> Result<Self, ServiceError> {
        config.validate()?;
        let http = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(std::time::Duration::from_secs(3))
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|_| ServiceError::Configuration)?;
        // The configured issuer is the authority; discovery must agree with it.
        let discovery_http = |request| {
            let config = config.clone();
            let http = http.clone();
            async move { discovery_request(&config, &http, request).await }
        };
        let metadata = CoreProviderMetadata::discover_async(
            IssuerUrl::new(config.issuer.clone()).map_err(|_| ServiceError::Configuration)?,
            &discovery_http,
        )
        .await
        .map_err(|_| ServiceError::IdentityUnavailable)?;
        let issuer_url = config.validate_endpoint(&config.issuer)?;
        let jwks_url = config.validate_endpoint(metadata.jwks_uri().as_str())?;
        if issuer_url.origin() != jwks_url.origin() {
            return Err(ServiceError::Configuration);
        }
        let keys: JwkSet = bounded_json(
            http.get(jwks_url.clone())
                .send()
                .await
                .map_err(|_| ServiceError::IdentityUnavailable)?,
        )
        .await?;
        if keys.keys.is_empty() || keys.keys.len() > 32 {
            return Err(ServiceError::IdentityUnavailable);
        }
        let secret_bytes =
            super::config::read_bounded_regular(&config.introspection_secret_file, 4096)?;
        if secret_bytes.is_empty() || secret_bytes.len() > 4096 {
            return Err(ServiceError::Configuration);
        }
        let secret = String::from_utf8(secret_bytes).map_err(|_| ServiceError::Configuration)?;
        if secret.trim() != secret || secret.trim().is_empty() {
            return Err(ServiceError::Configuration);
        }
        let introspection_endpoint = format!(
            "{}/protocol/openid-connect/token/introspect",
            config.issuer.trim_end_matches('/')
        );
        config.validate_endpoint(&introspection_endpoint)?;
        Ok(Self {
            config,
            http,
            keys: Arc::new(Mutex::new(KeyCache {
                keys,
                loaded_at: Instant::now(),
                attempted_at: None,
                refresh_failed: false,
            })),
            jwks_endpoint: jwks_url.to_string(),
            introspection_endpoint,
            secret: Arc::new(secret),
        })
    }

    pub async fn verify(&self, bearer: &str) -> Result<Principal, ServiceError> {
        let principal = self.verify_cached_signature(bearer).await?;
        // Per-request introspection makes Keycloak logout/revoke effective for subsequent requests.
        let response = self
            .http
            .post(&self.introspection_endpoint)
            .basic_auth(
                &self.config.introspection_client_id,
                Some(self.secret.as_str()),
            )
            .form(&[("token", bearer), ("token_type_hint", "access_token")])
            .send()
            .await
            .map_err(|_| ServiceError::IdentityUnavailable)?;
        let status: Introspection = bounded_json(response).await?;
        if !status.active
            || status.sub.as_deref() != Some(principal.subject.as_str())
            || !status.exp.is_some_and(|expiry| expiry > now_seconds())
        {
            return Err(ServiceError::Unauthorized);
        }
        Ok(principal)
    }

    async fn verify_cached_signature(&self, bearer: &str) -> Result<Principal, ServiceError> {
        if bearer.len() > 16384 {
            return Err(ServiceError::Unauthorized);
        }
        let header = decode_header(bearer).map_err(|_| ServiceError::Unauthorized)?;
        if header.alg != Algorithm::RS256 {
            return Err(ServiceError::Unauthorized);
        }
        let kid = header.kid.ok_or(ServiceError::Unauthorized)?;
        // Holding the async mutex through refresh coalesces concurrent unknown-key requests.
        let mut cache = self.keys.lock().await;
        let expired = cache.loaded_at.elapsed() >= KEY_TTL;
        if expired || cache.refresh_failed || cache.keys.find(&kid).is_none() {
            if cache
                .attempted_at
                .is_some_and(|attempt| attempt.elapsed() < REFRESH_BACKOFF)
            {
                return Err(if expired || cache.refresh_failed {
                    ServiceError::IdentityUnavailable
                } else {
                    ServiceError::Unauthorized
                });
            }
            cache.attempted_at = Some(Instant::now());
            cache.refresh_failed = true;
            let response = self
                .http
                .get(&self.jwks_endpoint)
                .send()
                .await
                .map_err(|_| ServiceError::IdentityUnavailable)?;
            let keys: JwkSet = bounded_json(response).await?;
            if keys.keys.is_empty() || keys.keys.len() > 32 {
                return Err(ServiceError::IdentityUnavailable);
            }
            cache.keys = keys;
            cache.loaded_at = Instant::now();
            cache.refresh_failed = false;
        }
        verify_signature(
            bearer,
            &cache.keys,
            &self.config.issuer,
            &self.config.audience,
        )
    }
}

async fn discovery_request(
    config: &ServiceConfig,
    http: &reqwest::Client,
    request: openidconnect::HttpRequest,
) -> Result<openidconnect::HttpResponse, ServiceError> {
    let (parts, body) = request.into_parts();
    if config.validate_endpoint(&parts.uri.to_string())?.origin()
        != config.validate_endpoint(&config.issuer)?.origin()
    {
        return Err(ServiceError::Configuration);
    }
    let mut response = http
        .request(parts.method, parts.uri.to_string())
        .headers(parts.headers)
        .body(body)
        .send()
        .await
        .map_err(|_| ServiceError::IdentityUnavailable)?;
    let mut builder = openidconnect::http::Response::builder().status(response.status());
    *builder
        .headers_mut()
        .ok_or(ServiceError::IdentityUnavailable)? = response.headers().clone();
    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| ServiceError::IdentityUnavailable)?
    {
        if bytes.len() + chunk.len() > 262144 {
            return Err(ServiceError::IdentityUnavailable);
        }
        bytes.extend_from_slice(&chunk);
    }
    builder
        .body(bytes)
        .map_err(|_| ServiceError::IdentityUnavailable)
}

fn verify_signature(
    bearer: &str,
    keys: &JwkSet,
    issuer: &str,
    audience: &str,
) -> Result<Principal, ServiceError> {
    if bearer.len() > 16384 {
        return Err(ServiceError::Unauthorized);
    }
    let header = decode_header(bearer).map_err(|_| ServiceError::Unauthorized)?;
    if header.alg != Algorithm::RS256 {
        return Err(ServiceError::Unauthorized);
    }
    let key = keys
        .find(header.kid.as_deref().ok_or(ServiceError::Unauthorized)?)
        .ok_or(ServiceError::Unauthorized)?;
    let key = DecodingKey::from_jwk(key).map_err(|_| ServiceError::Unauthorized)?;
    let mut policy = Validation::new(Algorithm::RS256);
    policy.set_issuer(&[issuer]);
    policy.set_audience(&[audience]);
    policy.set_required_spec_claims(&["sub", "iss", "aud", "exp"]);
    policy.leeway = 0;
    policy.validate_nbf = true;
    let claims = decode::<Claims>(bearer, &key, &policy)
        .map_err(|_| ServiceError::Unauthorized)?
        .claims;
    if claims.sub.is_empty()
        || claims.sub.len() > 256
        || claims.typ != "Bearer"
        || claims.exp <= now_seconds()
    {
        return Err(ServiceError::Unauthorized);
    }
    Ok(Principal {
        issuer: claims.iss,
        subject: claims.sub,
    })
}

async fn bounded_json<T: serde::de::DeserializeOwned>(
    mut response: reqwest::Response,
) -> Result<T, ServiceError> {
    if !response.status().is_success() {
        return Err(ServiceError::IdentityUnavailable);
    }
    let mut body = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| ServiceError::IdentityUnavailable)?
    {
        if body.len() + chunk.len() > 262144 {
            return Err(ServiceError::IdentityUnavailable);
        }
        body.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&body).map_err(|_| ServiceError::IdentityUnavailable)
}

pub fn now_seconds() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
