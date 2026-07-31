const USERNAME_MAXIMUM: usize = 24;
const PASSWORD_MAXIMUM: usize = 128;
const EMAIL_MAXIMUM: usize = 254;
const SECOND_FACTOR_MAXIMUM: usize = 14;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthMode {
    Login,
    Register,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthScreen {
    SignIn,
    PasswordResetRequest,
    ResetPassword,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthInputField {
    Username,
    Password,
    Email,
    SecondFactor,
    ForgotUsername,
    NewPassword,
    PasswordConfirmation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthStatus {
    Idle,
    TwoFactorRequired,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PasswordResetRequestOutcome {
    Sent,
    OpaqueFailure,
    RateLimited,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PasswordResetRequestStatus {
    Idle,
    Sent,
    RateLimited,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthCompletion {
    Authenticated,
    TwoFactorRequired,
    PasswordResetSucceeded,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthSecondFactor {
    pub code: String,
    pub recovery_code: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuthFlowEffect {
    Login {
        username: String,
        password: String,
        second_factor: AuthSecondFactor,
    },
    Register {
        username: String,
        password: String,
        email: String,
    },
    RequestPasswordReset {
        username: String,
    },
    ResetPassword {
        token: String,
        password: String,
    },
    NavigateToModeSelection,
    NavigateToRealmDirectory,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuthFlowError {
    InvalidScreen {
        action: &'static str,
        expected: AuthScreen,
        actual: AuthScreen,
    },
    Required {
        field: AuthInputField,
    },
    InputTooLong {
        field: AuthInputField,
        maximum: usize,
    },
    InvalidSignupEmail,
    PasswordConfirmationMismatch,
    MissingResetToken,
}

pub struct AuthFlow {
    mode: AuthMode,
    screen: AuthScreen,
    username: String,
    password: String,
    email: String,
    second_factor_input: String,
    two_factor_visible: bool,
    status: AuthStatus,
    forgot_username: String,
    password_reset_request_status: PasswordResetRequestStatus,
    reset_token: String,
    new_password: String,
    password_confirmation: String,
}

impl Default for AuthFlow {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthFlow {
    pub fn new() -> Self {
        Self {
            mode: AuthMode::Login,
            screen: AuthScreen::SignIn,
            username: String::new(),
            password: String::new(),
            email: String::new(),
            second_factor_input: String::new(),
            two_factor_visible: false,
            status: AuthStatus::Idle,
            forgot_username: String::new(),
            password_reset_request_status: PasswordResetRequestStatus::Idle,
            reset_token: String::new(),
            new_password: String::new(),
            password_confirmation: String::new(),
        }
    }

    pub const fn mode(&self) -> AuthMode {
        self.mode
    }

    pub const fn screen(&self) -> AuthScreen {
        self.screen
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub const fn two_factor_visible(&self) -> bool {
        self.two_factor_visible
    }

    pub const fn status(&self) -> AuthStatus {
        self.status
    }

    pub const fn password_reset_request_status(&self) -> PasswordResetRequestStatus {
        self.password_reset_request_status
    }

    pub fn set_auth_mode(&mut self, mode: AuthMode) {
        self.mode = mode;
        self.two_factor_visible = false;
        self.second_factor_input.clear();
        self.status = AuthStatus::Idle;
    }

    pub fn toggle_auth_mode(&mut self) {
        self.set_auth_mode(match self.mode {
            AuthMode::Login => AuthMode::Register,
            AuthMode::Register => AuthMode::Login,
        });
    }

    pub fn set_username(&mut self, value: impl Into<String>) -> Result<(), AuthFlowError> {
        self.require_screen("set_username", AuthScreen::SignIn)?;
        set_bounded(
            &mut self.username,
            value.into(),
            AuthInputField::Username,
            USERNAME_MAXIMUM,
        )
    }

    pub fn set_password(&mut self, value: impl Into<String>) -> Result<(), AuthFlowError> {
        self.require_screen("set_password", AuthScreen::SignIn)?;
        set_bounded(
            &mut self.password,
            value.into(),
            AuthInputField::Password,
            PASSWORD_MAXIMUM,
        )
    }

    pub fn set_email(&mut self, value: impl Into<String>) -> Result<(), AuthFlowError> {
        self.require_screen("set_email", AuthScreen::SignIn)?;
        set_bounded(
            &mut self.email,
            value.into(),
            AuthInputField::Email,
            EMAIL_MAXIMUM,
        )
    }

    pub fn set_second_factor_input(&mut self, value: impl AsRef<str>) -> Result<(), AuthFlowError> {
        self.require_screen("set_second_factor_input", AuthScreen::SignIn)?;
        let normalized = normalize_auth_code_input(value.as_ref());
        set_bounded(
            &mut self.second_factor_input,
            normalized,
            AuthInputField::SecondFactor,
            SECOND_FACTOR_MAXIMUM,
        )
    }

    pub fn submit_auth(&self) -> Result<AuthFlowEffect, AuthFlowError> {
        self.require_screen("submit_auth", AuthScreen::SignIn)?;
        require_nonempty(&self.username, AuthInputField::Username)?;
        require_nonempty(&self.password, AuthInputField::Password)?;

        let username = self.username.trim().to_string();
        match self.mode {
            AuthMode::Login => Ok(AuthFlowEffect::Login {
                username,
                password: self.password.clone(),
                second_factor: if self.two_factor_visible {
                    classify_auth_code(&self.second_factor_input)
                } else {
                    AuthSecondFactor {
                        code: String::new(),
                        recovery_code: String::new(),
                    }
                },
            }),
            AuthMode::Register => {
                require_nonempty(&self.email, AuthInputField::Email)?;
                let email = self.email.trim();
                if !valid_signup_email(email) {
                    return Err(AuthFlowError::InvalidSignupEmail);
                }
                Ok(AuthFlowEffect::Register {
                    username,
                    password: self.password.clone(),
                    email: email.to_string(),
                })
            }
        }
    }

    pub fn complete_auth(&mut self, completion: AuthCompletion) -> Option<AuthFlowEffect> {
        match completion {
            AuthCompletion::Authenticated => {
                self.password.clear();
                self.second_factor_input.clear();
                self.two_factor_visible = false;
                self.status = AuthStatus::Idle;
                Some(AuthFlowEffect::NavigateToRealmDirectory)
            }
            AuthCompletion::TwoFactorRequired => {
                self.two_factor_visible = true;
                self.status = AuthStatus::TwoFactorRequired;
                None
            }
            AuthCompletion::PasswordResetSucceeded => {
                self.clear_reset_password();
                self.screen = AuthScreen::SignIn;
                self.status = AuthStatus::Idle;
                None
            }
        }
    }

    pub fn open_password_reset_request(&mut self) {
        self.screen = AuthScreen::PasswordResetRequest;
        self.forgot_username.clear();
        self.password_reset_request_status = PasswordResetRequestStatus::Idle;
    }

    pub fn set_forgot_username(&mut self, value: impl Into<String>) -> Result<(), AuthFlowError> {
        self.require_screen("set_forgot_username", AuthScreen::PasswordResetRequest)?;
        set_bounded(
            &mut self.forgot_username,
            value.into(),
            AuthInputField::ForgotUsername,
            USERNAME_MAXIMUM,
        )
    }

    pub fn submit_password_reset_request(&self) -> Result<AuthFlowEffect, AuthFlowError> {
        self.require_screen(
            "submit_password_reset_request",
            AuthScreen::PasswordResetRequest,
        )?;
        require_nonempty(&self.forgot_username, AuthInputField::ForgotUsername)?;
        Ok(AuthFlowEffect::RequestPasswordReset {
            username: self.forgot_username.trim().to_string(),
        })
    }

    pub fn complete_password_reset_request(&mut self, outcome: PasswordResetRequestOutcome) {
        self.password_reset_request_status = match outcome {
            PasswordResetRequestOutcome::Sent | PasswordResetRequestOutcome::OpaqueFailure => {
                PasswordResetRequestStatus::Sent
            }
            PasswordResetRequestOutcome::RateLimited => PasswordResetRequestStatus::RateLimited,
        };
    }

    pub fn open_reset_password(&mut self, token: impl Into<String>) -> Result<(), AuthFlowError> {
        let token = token.into();
        if token.is_empty() {
            return Err(AuthFlowError::MissingResetToken);
        }
        self.screen = AuthScreen::ResetPassword;
        self.reset_token = token;
        self.new_password.clear();
        self.password_confirmation.clear();
        Ok(())
    }

    pub fn set_new_password(&mut self, value: impl Into<String>) -> Result<(), AuthFlowError> {
        self.require_screen("set_new_password", AuthScreen::ResetPassword)?;
        set_bounded(
            &mut self.new_password,
            value.into(),
            AuthInputField::NewPassword,
            PASSWORD_MAXIMUM,
        )
    }

    pub fn set_password_confirmation(
        &mut self,
        value: impl Into<String>,
    ) -> Result<(), AuthFlowError> {
        self.require_screen("set_password_confirmation", AuthScreen::ResetPassword)?;
        set_bounded(
            &mut self.password_confirmation,
            value.into(),
            AuthInputField::PasswordConfirmation,
            PASSWORD_MAXIMUM,
        )
    }

    pub fn submit_reset_password(&self) -> Result<AuthFlowEffect, AuthFlowError> {
        self.require_screen("submit_reset_password", AuthScreen::ResetPassword)?;
        require_nonempty(&self.new_password, AuthInputField::NewPassword)?;
        require_nonempty(
            &self.password_confirmation,
            AuthInputField::PasswordConfirmation,
        )?;
        if self.new_password != self.password_confirmation {
            return Err(AuthFlowError::PasswordConfirmationMismatch);
        }
        Ok(AuthFlowEffect::ResetPassword {
            token: self.reset_token.clone(),
            password: self.new_password.clone(),
        })
    }

    pub fn back(&mut self) -> Result<Option<AuthFlowEffect>, AuthFlowError> {
        match self.screen {
            AuthScreen::SignIn => {
                self.status = AuthStatus::Idle;
                Ok(Some(AuthFlowEffect::NavigateToModeSelection))
            }
            AuthScreen::PasswordResetRequest => {
                self.screen = AuthScreen::SignIn;
                Ok(None)
            }
            AuthScreen::ResetPassword => {
                self.clear_reset_password();
                self.screen = AuthScreen::SignIn;
                Ok(None)
            }
        }
    }

    fn clear_reset_password(&mut self) {
        self.reset_token.clear();
        self.new_password.clear();
        self.password_confirmation.clear();
    }

    fn require_screen(
        &self,
        action: &'static str,
        expected: AuthScreen,
    ) -> Result<(), AuthFlowError> {
        if self.screen == expected {
            Ok(())
        } else {
            Err(AuthFlowError::InvalidScreen {
                action,
                expected,
                actual: self.screen,
            })
        }
    }
}

pub fn normalize_auth_code_input(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed
        .chars()
        .all(|character| character.is_ascii_digit() || character.is_whitespace())
    {
        trimmed
            .chars()
            .filter(char::is_ascii_digit)
            .take(6)
            .collect()
    } else {
        trimmed.to_string()
    }
}

pub fn classify_auth_code(raw: &str) -> AuthSecondFactor {
    let trimmed = raw.trim();
    let compact = trimmed
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();
    if compact.len() == 6 && compact.bytes().all(|byte| byte.is_ascii_digit()) {
        AuthSecondFactor {
            code: compact,
            recovery_code: String::new(),
        }
    } else {
        AuthSecondFactor {
            code: String::new(),
            recovery_code: trimmed.to_string(),
        }
    }
}

fn set_bounded(
    destination: &mut String,
    value: String,
    field: AuthInputField,
    maximum: usize,
) -> Result<(), AuthFlowError> {
    if value.encode_utf16().count() > maximum {
        return Err(AuthFlowError::InputTooLong { field, maximum });
    }
    *destination = value;
    Ok(())
}

fn require_nonempty(value: &str, field: AuthInputField) -> Result<(), AuthFlowError> {
    if value.is_empty() {
        Err(AuthFlowError::Required { field })
    } else {
        Ok(())
    }
}

fn valid_signup_email(value: &str) -> bool {
    let Some((local, domain)) = value.split_once('@') else {
        return false;
    };
    let Some((domain_prefix, top_level_domain)) = domain.rsplit_once('.') else {
        return false;
    };
    !local.is_empty()
        && !domain.is_empty()
        && !domain.contains('@')
        && !domain_prefix.is_empty()
        && !top_level_domain.is_empty()
        && !value.chars().any(|character| character.is_whitespace())
}
