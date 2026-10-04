use super::AccountError;
use axum::{
    extract::{RawQuery, State},
    http::{header, HeaderMap, StatusCode},
    routing::get,
    Router,
};
use openidconnect::{url::form_urlencoded, AuthorizationCode, CsrfToken};
use std::{future::IntoFuture, sync::Arc};
use tokio::sync::{oneshot, Mutex};

#[derive(Clone)]
struct CallbackState {
    expected: CsrfToken,
    host: String,
    sender: Arc<Mutex<Option<oneshot::Sender<Result<AuthorizationCode, AccountError>>>>>,
}

pub async fn receive(
    listener: tokio::net::TcpListener,
    expected: CsrfToken,
) -> Result<AuthorizationCode, AccountError> {
    let (sender, receiver) = oneshot::channel();
    let state = CallbackState {
        expected,
        host: listener
            .local_addr()
            .map_err(|_| AccountError::Callback)?
            .to_string(),
        sender: Arc::new(Mutex::new(Some(sender))),
    };
    let router = Router::new()
        .route("/callback", get(callback))
        .with_state(state);
    let (shutdown, stopping) = oneshot::channel();
    let server = axum::serve(listener, router)
        .with_graceful_shutdown(async {
            let _ = stopping.await;
        })
        .into_future();
    tokio::pin!(server);
    let outcome = tokio::select! {
        result = receiver => result.map_err(|_| AccountError::Cancelled)?,
        _ = &mut server => return Err(AccountError::Callback),
        _ = tokio::time::sleep(std::time::Duration::from_secs(120)) => Err(AccountError::Timeout),
    };
    let _ = shutdown.send(());
    let _ = tokio::time::timeout(std::time::Duration::from_secs(1), &mut server).await;
    outcome
}

async fn callback(
    State(state): State<CallbackState>,
    headers: HeaderMap,
    RawQuery(query): RawQuery,
) -> (StatusCode, &'static str) {
    if headers.contains_key(header::ORIGIN)
        || headers.get(header::HOST).and_then(|v| v.to_str().ok()) != Some(state.host.as_str())
    {
        return (StatusCode::BAD_REQUEST, "Invalid callback");
    }
    let Ok(result) = parse(query.as_deref().unwrap_or(""), &state.expected) else {
        return (StatusCode::BAD_REQUEST, "Invalid callback");
    };
    let Some(sender) = state.sender.lock().await.take() else {
        return (StatusCode::GONE, "Callback completed");
    };
    let _ = sender.send(result);
    (
        StatusCode::OK,
        "Authentication completed. You can return to Zircon Hub.",
    )
}

fn parse(
    query: &str,
    expected: &CsrfToken,
) -> Result<Result<AuthorizationCode, AccountError>, AccountError> {
    if query.len() > 8192 {
        return Err(AccountError::Callback);
    }
    let pairs = form_urlencoded::parse(query.as_bytes()).collect::<Vec<_>>();
    let unique = |name: &str| {
        let values = pairs
            .iter()
            .filter(|(key, _)| key == name)
            .map(|(_, value)| value.to_string())
            .collect::<Vec<_>>();
        if values.len() == 1 {
            Some(values[0].clone())
        } else {
            None
        }
    };
    let state = unique("state").ok_or(AccountError::Callback)?;
    if CsrfToken::new(state) != *expected {
        return Err(AccountError::Callback);
    }
    if unique("error").is_some() {
        return Ok(Err(AccountError::Cancelled));
    }
    let code = unique("code")
        .filter(|code| !code.is_empty())
        .ok_or(AccountError::Callback)?;
    Ok(Ok(AuthorizationCode::new(code)))
}

#[cfg(test)]
#[path = "tests/callback.rs"]
mod tests;
