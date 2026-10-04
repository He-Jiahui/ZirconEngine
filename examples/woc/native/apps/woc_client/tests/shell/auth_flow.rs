use woc_client::{
    AuthCompletion, AuthFlow, AuthFlowEffect, AuthFlowError, AuthInputField, AuthMode,
    AuthRequestId, AuthScreen, AuthStatus, PasswordResetRequestOutcome, PasswordResetRequestStatus,
};

fn expect_auth_error<T>(result: Result<T, AuthFlowError>, message: &str) -> AuthFlowError {
    match result {
        Ok(_) => panic!("{message}"),
        Err(error) => error,
    }
}

fn request_id(effect: &AuthFlowEffect) -> AuthRequestId {
    effect
        .request_id()
        .expect("authentication request effects must carry an identity")
}

#[test]
fn login_trims_the_username_and_keeps_the_password_as_the_host_payload() {
    let mut flow = AuthFlow::new();
    flow.set_username("  Vale  ").expect("username input");
    flow.set_password("correct horse").expect("password input");

    match flow.submit_auth().expect("login request") {
        AuthFlowEffect::Login {
            username,
            password,
            second_factor,
            ..
        } => {
            assert_eq!(username, "Vale");
            assert_eq!(password, "correct horse");
            assert_eq!(second_factor.code, "");
            assert_eq!(second_factor.recovery_code, "");
        }
        AuthFlowEffect::Register { .. } => panic!("login mode must not emit registration"),
        _ => panic!("login submit must ask the host to authenticate"),
    }
}

#[test]
fn registration_requires_the_target_signup_email_shape_but_not_server_account_rules() {
    let mut flow = AuthFlow::new();
    flow.set_auth_mode(AuthMode::Register);
    flow.set_username("new_player").expect("username input");
    flow.set_password("secret").expect("password input");

    assert_eq!(
        expect_auth_error(flow.submit_auth(), "signup email is required"),
        AuthFlowError::Required {
            field: AuthInputField::Email
        }
    );

    flow.set_email("invalid").expect("email input");
    assert_eq!(
        expect_auth_error(flow.submit_auth(), "native UI mirrors type=email"),
        AuthFlowError::InvalidSignupEmail
    );

    flow.set_email("  new@example.com ").expect("email input");
    match flow.submit_auth().expect("registration request") {
        AuthFlowEffect::Register {
            username,
            password,
            email,
            ..
        } => {
            assert_eq!(username, "new_player");
            assert_eq!(password, "secret");
            assert_eq!(email, "new@example.com");
        }
        _ => panic!("register mode must emit registration only"),
    }
}

#[test]
fn registration_does_not_accept_a_login_only_two_factor_completion() {
    let mut flow = AuthFlow::new();
    flow.set_auth_mode(AuthMode::Register);
    flow.set_username("new_player").expect("username input");
    flow.set_password("secret").expect("password input");
    flow.set_email("new@example.com").expect("email input");

    let request = request_id(&flow.submit_auth().expect("registration request"));
    assert!(flow
        .complete_auth(request, AuthCompletion::TwoFactorRequired)
        .is_none());
    assert!(!flow.two_factor_visible());
    assert_eq!(flow.status(), AuthStatus::Idle);
}

#[test]
fn server_two_factor_challenge_replays_credentials_with_a_normalized_totp_code() {
    let mut flow = AuthFlow::new();
    flow.set_username("Vale").expect("username input");
    flow.set_password("secret").expect("password input");
    let initial_request = request_id(&flow.submit_auth().expect("initial login"));

    assert!(
        flow.complete_auth(initial_request, AuthCompletion::TwoFactorRequired)
            .is_none(),
        "a challenge remains on the authentication screen"
    );
    assert!(flow.two_factor_visible());
    assert_eq!(flow.status(), AuthStatus::TwoFactorRequired);

    flow.set_second_factor_input(" 1 2 3 4 5 6 ")
        .expect("numeric code input");
    match flow.submit_auth().expect("follow-up login") {
        AuthFlowEffect::Login { second_factor, .. } => {
            assert_eq!(second_factor.code, "123456");
            assert_eq!(second_factor.recovery_code, "");
        }
        _ => panic!("two-factor follow-up must remain a login request"),
    }
}

#[test]
fn changing_primary_login_credentials_invalidates_the_old_two_factor_challenge() {
    let mut flow = AuthFlow::new();
    flow.set_username("Vale").expect("username input");
    flow.set_password("first-secret").expect("password input");
    let request = request_id(&flow.submit_auth().expect("initial login"));
    flow.complete_auth(request, AuthCompletion::TwoFactorRequired);
    flow.set_second_factor_input("123456")
        .expect("two-factor input");

    flow.set_password("replacement-secret")
        .expect("replacement password");
    assert!(!flow.two_factor_visible());
    assert_eq!(flow.status(), AuthStatus::Idle);

    match flow.submit_auth().expect("new login") {
        AuthFlowEffect::Login { second_factor, .. } => {
            assert_eq!(second_factor.code.as_str(), "");
            assert_eq!(second_factor.recovery_code.as_str(), "");
        }
        _ => panic!("replacement credentials must emit a login request"),
    }
}

#[test]
fn non_totp_second_factor_is_sent_as_a_trimmed_recovery_code() {
    let mut flow = AuthFlow::new();
    flow.set_username("Vale").expect("username input");
    flow.set_password("secret").expect("password input");
    let initial_request = request_id(&flow.submit_auth().expect("initial login"));
    flow.complete_auth(initial_request, AuthCompletion::TwoFactorRequired);
    flow.set_second_factor_input("  restore-A1 ")
        .expect("recovery code input");

    match flow.submit_auth().expect("recovery login") {
        AuthFlowEffect::Login { second_factor, .. } => {
            assert_eq!(second_factor.code, "");
            assert_eq!(second_factor.recovery_code, "restore-A1");
        }
        _ => panic!("recovery code must remain a login request"),
    }
}

#[test]
fn password_reset_request_keeps_account_enumeration_safe_except_for_rate_limit() {
    let mut flow = AuthFlow::new();
    flow.open_password_reset_request();
    flow.set_forgot_username("  Vale ")
        .expect("forgot username input");

    let first_request = flow
        .submit_password_reset_request()
        .expect("reset request effect");
    match &first_request {
        AuthFlowEffect::RequestPasswordReset { username, .. } => assert_eq!(username, "Vale"),
        _ => panic!("forgot-password submit must request a reset link"),
    }

    let opaque_request = request_id(&first_request);

    assert!(flow.complete_password_reset_request(
        opaque_request,
        PasswordResetRequestOutcome::OpaqueFailure,
    ));
    assert_eq!(
        flow.password_reset_request_status(),
        PasswordResetRequestStatus::Sent
    );

    let rate_limited_request =
        request_id(&flow.submit_password_reset_request().expect("retry reset"));
    assert!(flow.complete_password_reset_request(
        rate_limited_request,
        PasswordResetRequestOutcome::RateLimited,
    ));
    assert_eq!(
        flow.password_reset_request_status(),
        PasswordResetRequestStatus::RateLimited
    );
}

#[test]
fn reset_password_requires_matching_nonempty_fields_then_discards_the_token_on_success() {
    let mut flow = AuthFlow::new();
    flow.open_reset_password("one-time-token")
        .expect("host supplied reset token");

    assert_eq!(
        expect_auth_error(
            flow.submit_reset_password(),
            "empty password fields are local form errors",
        ),
        AuthFlowError::Required {
            field: AuthInputField::NewPassword
        }
    );

    flow.set_new_password("first").expect("new password input");
    flow.set_password_confirmation("second")
        .expect("confirmation input");
    assert_eq!(
        expect_auth_error(flow.submit_reset_password(), "confirmation must match"),
        AuthFlowError::PasswordConfirmationMismatch
    );

    flow.set_password_confirmation("first")
        .expect("matching confirmation");
    let reset_effect = flow.submit_reset_password().expect("reset effect");
    match &reset_effect {
        AuthFlowEffect::ResetPassword {
            token, password, ..
        } => {
            assert_eq!(token.as_str(), "one-time-token");
            assert_eq!(password.as_str(), "first");
        }
        _ => panic!("matching reset form must ask the host to update the password"),
    }

    let reset_request = request_id(&reset_effect);
    assert!(flow
        .complete_auth(reset_request, AuthCompletion::PasswordResetSucceeded)
        .is_none());
    assert_eq!(flow.screen(), AuthScreen::SignIn);
}

#[test]
fn back_routes_only_the_sign_in_screen_to_mode_selection() {
    let mut flow = AuthFlow::new();
    assert!(matches!(
        flow.back().expect("sign-in back"),
        Some(AuthFlowEffect::NavigateToModeSelection)
    ));

    flow.open_password_reset_request();
    assert!(flow.back().expect("forgot back").is_none());
    assert_eq!(flow.screen(), AuthScreen::SignIn);
}

#[test]
fn leaving_sign_in_discards_login_credentials_and_two_factor_challenge() {
    let mut flow = AuthFlow::new();
    flow.set_username("Vale").expect("username input");
    flow.set_password("prior-password").expect("password input");
    let request = request_id(&flow.submit_auth().expect("initial login"));
    flow.complete_auth(request, AuthCompletion::TwoFactorRequired);
    flow.set_second_factor_input("123456")
        .expect("two-factor input");

    assert!(matches!(
        flow.back().expect("leave sign-in"),
        Some(AuthFlowEffect::NavigateToModeSelection)
    ));
    assert!(!flow.two_factor_visible());
    assert_eq!(flow.status(), AuthStatus::Idle);
    assert_eq!(
        flow.submit_auth()
            .expect_err("credentials must not survive navigation away from sign-in"),
        AuthFlowError::Required {
            field: AuthInputField::Password,
        }
    );
}

#[test]
fn bounded_input_rejection_does_not_replace_the_existing_safe_value() {
    let mut flow = AuthFlow::new();
    flow.set_email("player@example.com")
        .expect("valid email input");

    assert_eq!(
        expect_auth_error(
            flow.set_email("a".repeat(255)),
            "email maxlength mirrors the target field",
        ),
        AuthFlowError::InputTooLong {
            field: AuthInputField::Email,
            maximum: 254,
        }
    );
    assert_eq!(flow.email(), "player@example.com");
}

#[test]
fn whitespace_only_usernames_are_rejected_before_emitting_auth_requests() {
    let mut flow = AuthFlow::new();
    flow.set_username("  \t  ").expect("bounded username input");
    flow.set_password("secret").expect("password input");

    assert_eq!(
        flow.submit_auth()
            .expect_err("trimmed username must be required"),
        AuthFlowError::Required {
            field: AuthInputField::Username,
        }
    );

    flow.open_password_reset_request();
    flow.set_forgot_username(" \n ")
        .expect("bounded forgot username input");
    assert_eq!(
        flow.submit_password_reset_request()
            .expect_err("trimmed forgot username must be required"),
        AuthFlowError::Required {
            field: AuthInputField::ForgotUsername,
        }
    );
}

#[test]
fn authentication_effect_debug_redacts_passwords_tokens_and_second_factor_values() {
    let mut flow = AuthFlow::new();
    flow.set_username("Vale").expect("username input");
    flow.set_password("super-secret-password")
        .expect("password input");
    let initial_request = request_id(&flow.submit_auth().expect("initial login"));
    flow.complete_auth(initial_request, AuthCompletion::TwoFactorRequired);
    flow.set_second_factor_input("123456")
        .expect("two-factor input");

    let login_debug = format!("{:?}", flow.submit_auth().expect("login effect"));
    assert!(login_debug.contains("username: \"Vale\""));
    assert!(login_debug.contains("password: \"[REDACTED]\""));
    assert!(login_debug.contains("code: \"[REDACTED]\""));
    assert!(!login_debug.contains("super-secret-password"));
    assert!(!login_debug.contains("123456"));

    flow.set_auth_mode(AuthMode::Register);
    flow.set_email("vale@example.com").expect("email input");
    let register_debug = format!("{:?}", flow.submit_auth().expect("register effect"));
    assert!(register_debug.contains("email: \"vale@example.com\""));
    assert!(!register_debug.contains("super-secret-password"));

    flow.open_reset_password("one-time-reset-token")
        .expect("reset token");
    flow.set_new_password("replacement-secret")
        .expect("new password");
    flow.set_password_confirmation("replacement-secret")
        .expect("password confirmation");
    let reset_debug = format!("{:?}", flow.submit_reset_password().expect("reset effect"));
    assert!(reset_debug.contains("token: \"[REDACTED]\""));
    assert!(reset_debug.contains("password: \"[REDACTED]\""));
    assert!(!reset_debug.contains("one-time-reset-token"));
    assert!(!reset_debug.contains("replacement-secret"));
}

#[test]
fn recovery_code_debug_is_redacted_even_when_it_is_not_a_totp_code() {
    let second_factor = woc_client::classify_auth_code("restore-code-42");
    let debug = format!("{second_factor:?}");

    assert!(debug.contains("code: \"[REDACTED]\""));
    assert!(debug.contains("recovery_code: \"[REDACTED]\""));
    assert!(!debug.contains("restore-code-42"));
}

#[test]
fn auth_secret_exposes_only_a_borrow_and_clears_its_buffer() {
    let mut secret = woc_client::AuthSecret::from("ephemeral-secret");
    assert_eq!(secret.as_str(), "ephemeral-secret");
    assert_eq!(format!("{secret:?}"), "\"[REDACTED]\"");
    assert_eq!(secret.to_string(), "[REDACTED]");

    secret.clear();
    assert!(secret.is_empty());
    assert_eq!(secret.as_str(), "");
}

#[test]
fn reset_tokens_are_bounded_and_cannot_carry_whitespace_or_control_data() {
    let mut flow = AuthFlow::new();
    assert_eq!(
        flow.open_reset_password("token with spaces")
            .expect_err("token whitespace must be rejected"),
        AuthFlowError::InvalidResetToken
    );
    assert_eq!(
        flow.open_reset_password("token\nwith-control")
            .expect_err("token controls must be rejected"),
        AuthFlowError::InvalidResetToken
    );
    assert_eq!(
        flow.open_reset_password(" ")
            .expect_err("empty token must retain the required error"),
        AuthFlowError::MissingResetToken
    );
    let oversized = "x".repeat(4097);
    assert_eq!(
        flow.open_reset_password(oversized)
            .expect_err("oversized token must be rejected"),
        AuthFlowError::InvalidResetToken
    );
}

#[test]
fn password_reset_request_discards_login_credentials_and_two_factor_challenge() {
    let mut flow = AuthFlow::new();
    flow.set_username("Vale").expect("username input");
    flow.set_password("prior-password").expect("password input");
    let request = request_id(&flow.submit_auth().expect("initial login"));
    flow.complete_auth(request, AuthCompletion::TwoFactorRequired);
    flow.set_second_factor_input("123456")
        .expect("two-factor input");

    flow.open_password_reset_request();

    assert_eq!(flow.screen(), AuthScreen::PasswordResetRequest);
    assert!(!flow.two_factor_visible());
    assert_eq!(flow.status(), AuthStatus::Idle);
    flow.back().expect("return to sign in");
    assert_eq!(
        flow.submit_auth()
            .expect_err("the old login password must not survive reset navigation"),
        AuthFlowError::Required {
            field: AuthInputField::Password,
        }
    );
}

#[test]
fn reset_password_entry_discards_login_credentials_and_two_factor_challenge() {
    let mut flow = AuthFlow::new();
    flow.set_username("Vale").expect("username input");
    flow.set_password("prior-password").expect("password input");
    let request = request_id(&flow.submit_auth().expect("initial login"));
    flow.complete_auth(request, AuthCompletion::TwoFactorRequired);
    flow.set_second_factor_input("123456")
        .expect("two-factor input");

    flow.open_reset_password("fresh-reset-token")
        .expect("reset token");

    assert_eq!(flow.screen(), AuthScreen::ResetPassword);
    assert!(!flow.two_factor_visible());
    assert_eq!(flow.status(), AuthStatus::Idle);
    flow.back().expect("return to sign in");
    assert_eq!(
        flow.submit_auth()
            .expect_err("the old login password must not survive reset entry"),
        AuthFlowError::Required {
            field: AuthInputField::Password,
        }
    );
}

#[test]
fn stale_auth_completion_cannot_override_a_newer_login_request() {
    let mut flow = AuthFlow::new();
    flow.set_username("Vale").expect("username input");
    flow.set_password("first-secret").expect("password input");
    let first = request_id(&flow.submit_auth().expect("first login"));

    flow.set_password("second-secret")
        .expect("replacement password");
    let second = request_id(&flow.submit_auth().expect("second login"));
    assert_ne!(first, second);

    assert!(flow
        .complete_auth(first, AuthCompletion::Authenticated)
        .is_none());
    assert!(!flow.two_factor_visible());
    assert_eq!(flow.status(), AuthStatus::Idle);

    assert!(flow
        .complete_auth(second, AuthCompletion::TwoFactorRequired)
        .is_none());
    assert!(flow.two_factor_visible());
    assert_eq!(flow.status(), AuthStatus::TwoFactorRequired);
}

#[test]
fn stale_password_reset_request_outcome_cannot_replace_the_newer_request_status() {
    let mut flow = AuthFlow::new();
    flow.open_password_reset_request();
    flow.set_forgot_username("Vale").expect("forgot username");
    let first = request_id(&flow.submit_password_reset_request().expect("first request"));

    flow.set_forgot_username("OtherVale")
        .expect("replacement username");
    let second = request_id(
        &flow
            .submit_password_reset_request()
            .expect("second request"),
    );
    assert_ne!(first, second);

    assert!(!flow.complete_password_reset_request(first, PasswordResetRequestOutcome::RateLimited));
    assert_eq!(
        flow.password_reset_request_status(),
        PasswordResetRequestStatus::Idle
    );

    assert!(
        flow.complete_password_reset_request(second, PasswordResetRequestOutcome::OpaqueFailure,)
    );
    assert_eq!(
        flow.password_reset_request_status(),
        PasswordResetRequestStatus::Sent
    );
}

#[test]
fn stale_or_wrong_kind_completion_cannot_reset_a_newer_password_change() {
    let mut flow = AuthFlow::new();
    flow.open_reset_password("first-token")
        .expect("host supplied reset token");
    flow.set_new_password("first-secret")
        .expect("new password input");
    flow.set_password_confirmation("first-secret")
        .expect("matching confirmation");
    let first = request_id(&flow.submit_reset_password().expect("first reset"));

    flow.set_new_password("second-secret")
        .expect("replacement password");
    flow.set_password_confirmation("second-secret")
        .expect("replacement confirmation");
    let second = request_id(&flow.submit_reset_password().expect("second reset"));
    assert_ne!(first, second);

    assert!(flow
        .complete_auth(first, AuthCompletion::PasswordResetSucceeded)
        .is_none());
    assert_eq!(flow.screen(), AuthScreen::ResetPassword);

    assert!(flow
        .complete_auth(second, AuthCompletion::TwoFactorRequired)
        .is_none());
    assert_eq!(flow.screen(), AuthScreen::ResetPassword);

    assert!(flow
        .complete_auth(second, AuthCompletion::PasswordResetSucceeded)
        .is_none());
    assert_eq!(flow.screen(), AuthScreen::SignIn);
}
