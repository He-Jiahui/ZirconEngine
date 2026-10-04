use super::*;
#[test]
fn callback_requires_unique_state_and_code() {
    let state = CsrfToken::new("expected".into());
    assert!(parse("state=expected&code=one", &state).unwrap().is_ok());
    for query in [
        "state=wrong&code=one",
        "state=expected&state=expected&code=one",
        "state=expected&code=one&code=two",
        "state=expected",
        "code=one",
    ] {
        assert!(parse(query, &state).is_err());
    }
    assert!(matches!(
        parse("state=expected&error=access_denied", &state),
        Ok(Err(AccountError::Cancelled))
    ));
}

#[tokio::test]
async fn invalid_callback_does_not_consume_the_valid_attempt() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let task = tokio::spawn(receive(listener, CsrfToken::new("expected".into())));
    let http = super::super::oidc::http().unwrap();
    let wrong = http
        .get(format!("http://{address}/callback?state=wrong&code=one"))
        .send()
        .await
        .unwrap();
    assert_eq!(wrong.status(), StatusCode::BAD_REQUEST);
    let valid = http
        .get(format!("http://{address}/callback?state=expected&code=one"))
        .send()
        .await
        .unwrap();
    assert_eq!(valid.status(), StatusCode::OK);
    assert_eq!(task.await.unwrap().unwrap().secret(), "one");
    assert!(http
        .get(format!("http://{address}/callback?state=expected&code=one"))
        .send()
        .await
        .is_err());
}
