use woc_client::{
    AuthCompletion, AuthFlow, AuthFlowEffect, AuthFlowError, AuthInputField, AuthMode, AuthScreen,
    AuthStatus, PasswordResetRequestOutcome, PasswordResetRequestStatus,
};

fn expect_auth_error<T>(result: Result<T, AuthFlowError>, message: &str) -> AuthFlowError {
    match result {
        Ok(_) => panic!("{message}"),
        Err(error) => error,
    }
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
        } => {
            assert_eq!(username, "new_player");
            assert_eq!(password, "secret");
            assert_eq!(email, "new@example.com");
        }
        _ => panic!("register mode must emit registration only"),
    }
}

#[test]
fn server_two_factor_challenge_replays_credentials_with_a_normalized_totp_code() {
    let mut flow = AuthFlow::new();
    flow.set_username("Vale").expect("username input");
    flow.set_password("secret").expect("password input");

    assert!(
        flow.complete_auth(AuthCompletion::TwoFactorRequired)
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
fn non_totp_second_factor_is_sent_as_a_trimmed_recovery_code() {
    let mut flow = AuthFlow::new();
    flow.set_username("Vale").expect("username input");
    flow.set_password("secret").expect("password input");
    flow.complete_auth(AuthCompletion::TwoFactorRequired);
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

    match flow
        .submit_password_reset_request()
        .expect("reset request effect")
    {
        AuthFlowEffect::RequestPasswordReset { username } => assert_eq!(username, "Vale"),
        _ => panic!("forgot-password submit must request a reset link"),
    }

    flow.complete_password_reset_request(PasswordResetRequestOutcome::OpaqueFailure);
    assert_eq!(
        flow.password_reset_request_status(),
        PasswordResetRequestStatus::Sent
    );

    flow.complete_password_reset_request(PasswordResetRequestOutcome::RateLimited);
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
    match flow.submit_reset_password().expect("reset effect") {
        AuthFlowEffect::ResetPassword { token, password } => {
            assert_eq!(token, "one-time-token");
            assert_eq!(password, "first");
        }
        _ => panic!("matching reset form must ask the host to update the password"),
    }

    assert!(flow
        .complete_auth(AuthCompletion::PasswordResetSucceeded)
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
