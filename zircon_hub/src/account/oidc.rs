use super::{callback, config::AccountConfig, credential::SavedSession, AccountError};
use openidconnect::{
    core::{CoreAuthenticationFlow, CoreClient, CoreProviderMetadata, CoreTokenResponse},
    reqwest, AccessTokenHash, ClientId, CsrfToken, IssuerUrl, Nonce, NonceVerifier,
    OAuth2TokenResponse, PkceCodeChallenge, RedirectUrl, RefreshToken, Scope, TokenResponse,
};

pub struct Authenticated {
    pub subject: String,
    pub name: String,
    pub nonce: String,
    pub tokens: CoreTokenResponse,
}

pub fn http() -> Result<reqwest::Client, AccountError> {
    reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(std::time::Duration::from_secs(3))
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|_| AccountError::ProviderUnavailable)
}

async fn provider_request(
    config: &AccountConfig,
    http: &reqwest::Client,
    request: openidconnect::HttpRequest,
) -> Result<openidconnect::HttpResponse, AccountError> {
    let (parts, body) = request.into_parts();
    if config.endpoint(&parts.uri.to_string())?.origin()
        != config.endpoint(&config.issuer)?.origin()
    {
        return Err(AccountError::Configuration);
    }
    let mut response = http
        .request(parts.method, parts.uri.to_string())
        .headers(parts.headers)
        .body(body)
        .send()
        .await
        .map_err(|_| AccountError::ProviderUnavailable)?;
    if response.status().is_server_error() {
        return Err(AccountError::ProviderUnavailable);
    }
    let mut builder = openidconnect::http::Response::builder().status(response.status());
    *builder
        .headers_mut()
        .ok_or(AccountError::ProviderUnavailable)? = response.headers().clone();
    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| AccountError::ProviderUnavailable)?
    {
        if bytes.len() + chunk.len() > 1048576 {
            return Err(AccountError::ProviderUnavailable);
        }
        bytes.extend_from_slice(&chunk);
    }
    builder
        .body(bytes)
        .map_err(|_| AccountError::ProviderUnavailable)
}

async fn metadata(
    config: &AccountConfig,
    http: &reqwest::Client,
) -> Result<CoreProviderMetadata, AccountError> {
    let metadata = CoreProviderMetadata::discover_async(
        IssuerUrl::new(config.issuer.clone()).map_err(|_| AccountError::Configuration)?,
        &|request| provider_request(config, http, request),
    )
    .await
    .map_err(|_| AccountError::ProviderUnavailable)?;
    let origin = config.endpoint(&config.issuer)?.origin();
    for endpoint in [
        metadata.authorization_endpoint().as_str(),
        metadata.jwks_uri().as_str(),
        metadata
            .token_endpoint()
            .ok_or(AccountError::Configuration)?
            .as_str(),
    ] {
        if config.endpoint(endpoint)?.origin() != origin {
            return Err(AccountError::Configuration);
        }
    }
    Ok(metadata)
}

pub async fn authenticate(
    config: &AccountConfig,
    saved: Option<&SavedSession>,
) -> Result<Authenticated, AccountError> {
    let http = http()?;
    let metadata = metadata(config, &http).await?;
    let transport = |request| provider_request(config, &http, request);
    let client =
        CoreClient::from_provider_metadata(metadata, ClientId::new(config.client_id.clone()), None)
            .set_redirect_uri(
                RedirectUrl::new(config.redirect()).map_err(|_| AccountError::Configuration)?,
            );
    let refresh_session = saved.is_some();
    let (tokens, nonce) = if let Some(saved) = saved {
        if saved.revoke_only {
            return Err(AccountError::SessionExpired);
        }
        if saved.issuer != config.issuer {
            return Err(AccountError::InvalidIdentity);
        }
        let token = RefreshToken::new(saved.refresh_token.clone());
        let tokens = client
            .exchange_refresh_token(&token)
            .map_err(|_| AccountError::Configuration)?
            .request_async(&transport)
            .await
            .map_err(refresh_error)?;
        (tokens, Nonce::new(saved.nonce.clone()))
    } else {
        let listener =
            tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, config.callback_port))
                .await
                .map_err(|_| AccountError::Callback)?;
        let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
        let (url, state, nonce) = client
            .authorize_url(
                CoreAuthenticationFlow::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            .add_scope(Scope::new("profile".into()))
            .set_pkce_challenge(challenge)
            .url();
        webbrowser::open(url.as_str()).map_err(|_| AccountError::Browser)?;
        let code = callback::receive(listener, state).await?;
        let tokens = client
            .exchange_code(code)
            .map_err(|_| AccountError::Configuration)?
            .set_pkce_verifier(verifier)
            .request_async(&transport)
            .await
            .map_err(|_| AccountError::InvalidIdentity)?;
        (tokens, nonce)
    };
    let id_token = tokens.id_token().ok_or(AccountError::InvalidIdentity)?;
    let verifier = client.id_token_verifier();
    let claims = if refresh_session {
        // OIDC Core 12.2 permits omission on refresh, but any returned nonce must match.
        id_token.claims(&verifier, |claim_nonce: Option<&Nonce>| {
            if claim_nonce.is_none() {
                Ok(())
            } else {
                (&nonce).verify(claim_nonce)
            }
        })
    } else {
        id_token.claims(&verifier, &nonce)
    }
    .map_err(|_| AccountError::InvalidIdentity)?;
    if let Some(expected) = claims.access_token_hash() {
        let actual = AccessTokenHash::from_token(
            tokens.access_token(),
            id_token
                .signing_alg()
                .map_err(|_| AccountError::InvalidIdentity)?,
            id_token
                .signing_key(&verifier)
                .map_err(|_| AccountError::InvalidIdentity)?,
        )
        .map_err(|_| AccountError::InvalidIdentity)?;
        if actual != *expected {
            return Err(AccountError::InvalidIdentity);
        }
    }
    let subject = claims.subject().as_str().to_owned();
    if subject.is_empty() || saved.is_some_and(|saved| saved.subject != subject) {
        return Err(AccountError::InvalidIdentity);
    }
    let name = claims
        .preferred_username()
        .map(|name| name.as_str().to_owned())
        .unwrap_or_else(|| subject.clone());
    Ok(Authenticated {
        subject,
        name,
        nonce: nonce.secret().clone(),
        tokens,
    })
}

fn refresh_error(
    error: openidconnect::RequestTokenError<
        AccountError,
        openidconnect::StandardErrorResponse<openidconnect::core::CoreErrorResponseType>,
    >,
) -> AccountError {
    match error {
        openidconnect::RequestTokenError::ServerResponse(response)
            if response.error().as_ref() == "invalid_grant" =>
        {
            AccountError::SessionExpired
        }
        _ => AccountError::ProviderUnavailable,
    }
}

pub async fn revoke(config: &AccountConfig, refresh_token: &str) -> Result<(), AccountError> {
    let endpoint = format!(
        "{}/protocol/openid-connect/logout",
        config.issuer.trim_end_matches('/')
    );
    config.endpoint(&endpoint)?;
    let response = http()?
        .post(endpoint)
        .form(&[
            ("client_id", config.client_id.as_str()),
            ("refresh_token", refresh_token),
        ])
        .send()
        .await
        .map_err(|_| AccountError::ProviderUnavailable)?;
    if !response.status().is_success() {
        return Err(AccountError::ProviderUnavailable);
    }
    Ok(())
}

#[cfg(test)]
#[path = "oidc/tests/cases.rs"]
mod tests;
