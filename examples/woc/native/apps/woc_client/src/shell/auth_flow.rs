use std::fmt;

const USERNAME_MAXIMUM: usize = 24;
const PASSWORD_MAXIMUM: usize = 128;
const EMAIL_MAXIMUM: usize = 254;
const SECOND_FACTOR_MAXIMUM: usize = 14;
const RESET_TOKEN_MAXIMUM: usize = 4096;

/// In-memory credential material with redacted diagnostics and best-effort
/// zeroing on clear/drop. The byte vector is kept UTF-8-valid by construction.
#[derive(Clone, PartialEq, Eq)]
pub struct AuthSecret(Vec<u8>);

impl AuthSecret {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into().into_bytes())
    }

    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.0).expect("AuthSecret must remain valid UTF-8")
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn clear(&mut self) {
        for byte in &mut self.0 {
            *byte = 0;
        }
        self.0.clear();
    }
}

impl Default for AuthSecret {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl From<String> for AuthSecret {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for AuthSecret {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl AsRef<str> for AuthSecret {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::ops::Deref for AuthSecret {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl PartialEq<str> for AuthSecret {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for AuthSecret {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<String> for AuthSecret {
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other
    }
}

impl fmt::Debug for AuthSecret {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("\"[REDACTED]\"")
    }
}

impl fmt::Display for AuthSecret {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("[REDACTED]")
    }
}

impl Drop for AuthSecret {
    fn drop(&mut self) {
        self.clear();
    }
}

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AuthRequestId(u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PendingAuthRequest {
    Login(AuthRequestId),
    Register(AuthRequestId),
    PasswordResetRequest(AuthRequestId),
    ResetPassword(AuthRequestId),
}

#[derive(Clone, PartialEq, Eq)]
pub struct AuthSecondFactor {
    pub code: AuthSecret,
    pub recovery_code: AuthSecret,
}

#[derive(Clone, PartialEq, Eq)]
pub enum AuthFlowEffect {
    Login {
        request_id: AuthRequestId,
        username: String,
        password: AuthSecret,
        second_factor: AuthSecondFactor,
    },
    Register {
        request_id: AuthRequestId,
        username: String,
        password: AuthSecret,
        email: String,
    },
    RequestPasswordReset {
        request_id: AuthRequestId,
        username: String,
    },
    ResetPassword {
        request_id: AuthRequestId,
        token: AuthSecret,
        password: AuthSecret,
    },
    NavigateToModeSelection,
    NavigateToRealmDirectory,
}

impl AuthFlowEffect {
    pub const fn request_id(&self) -> Option<AuthRequestId> {
        match self {
            Self::Login { request_id, .. }
            | Self::Register { request_id, .. }
            | Self::RequestPasswordReset { request_id, .. }
            | Self::ResetPassword { request_id, .. } => Some(*request_id),
            Self::NavigateToModeSelection | Self::NavigateToRealmDirectory => None,
        }
    }
}

struct RedactedSecret;

impl fmt::Debug for RedactedSecret {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("\"[REDACTED]\"")
    }
}

impl fmt::Debug for AuthSecondFactor {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AuthSecondFactor")
            .field("code", &RedactedSecret)
            .field("recovery_code", &RedactedSecret)
            .finish()
    }
}

impl fmt::Debug for AuthFlowEffect {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Login {
                request_id,
                username,
                password: _,
                second_factor,
            } => formatter
                .debug_struct("Login")
                .field("request_id", request_id)
                .field("username", username)
                .field("password", &RedactedSecret)
                .field("second_factor", second_factor)
                .finish(),
            Self::Register {
                request_id,
                username,
                password: _,
                email,
            } => formatter
                .debug_struct("Register")
                .field("request_id", request_id)
                .field("username", username)
                .field("password", &RedactedSecret)
                .field("email", email)
                .finish(),
            Self::RequestPasswordReset {
                request_id,
                username,
            } => formatter
                .debug_struct("RequestPasswordReset")
                .field("request_id", request_id)
                .field("username", username)
                .finish(),
            Self::ResetPassword {
                request_id,
                token: _,
                password: _,
            } => formatter
                .debug_struct("ResetPassword")
                .field("request_id", request_id)
                .field("token", &RedactedSecret)
                .field("password", &RedactedSecret)
                .finish(),
            Self::NavigateToModeSelection => formatter.write_str("NavigateToModeSelection"),
            Self::NavigateToRealmDirectory => formatter.write_str("NavigateToRealmDirectory"),
        }
    }
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
    InvalidResetToken,
    RequestIdExhausted,
}

pub struct AuthFlow {
    mode: AuthMode,
    screen: AuthScreen,
    username: String,
    password: AuthSecret,
    email: String,
    second_factor_input: AuthSecret,
    two_factor_visible: bool,
    status: AuthStatus,
    forgot_username: String,
    password_reset_request_status: PasswordResetRequestStatus,
    reset_token: AuthSecret,
    new_password: AuthSecret,
    password_confirmation: AuthSecret,
    next_request_id: u64,
    pending_request: Option<PendingAuthRequest>,
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
            password: AuthSecret::default(),
            email: String::new(),
            second_factor_input: AuthSecret::default(),
            two_factor_visible: false,
            status: AuthStatus::Idle,
            forgot_username: String::new(),
            password_reset_request_status: PasswordResetRequestStatus::Idle,
            reset_token: AuthSecret::default(),
            new_password: AuthSecret::default(),
            password_confirmation: AuthSecret::default(),
            next_request_id: 1,
            pending_request: None,
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
        self.invalidate_pending_request();
        self.mode = mode;
        self.clear_two_factor_challenge();
    }

    pub fn toggle_auth_mode(&mut self) {
        self.set_auth_mode(match self.mode {
            AuthMode::Login => AuthMode::Register,
            AuthMode::Register => AuthMode::Login,
        });
    }

    pub fn set_username(&mut self, value: impl Into<String>) -> Result<(), AuthFlowError> {
        self.require_screen("set_username", AuthScreen::SignIn)?;
        let result = set_bounded(
            &mut self.username,
            value.into(),
            AuthInputField::Username,
            USERNAME_MAXIMUM,
        );
        if result.is_ok() {
            self.invalidate_pending_request();
            self.clear_two_factor_challenge();
        }
        result
    }

    pub fn set_password(&mut self, value: impl Into<String>) -> Result<(), AuthFlowError> {
        self.require_screen("set_password", AuthScreen::SignIn)?;
        let result = set_secret_bounded(
            &mut self.password,
            value.into(),
            AuthInputField::Password,
            PASSWORD_MAXIMUM,
        );
        if result.is_ok() {
            self.invalidate_pending_request();
            self.clear_two_factor_challenge();
        }
        result
    }

    pub fn set_email(&mut self, value: impl Into<String>) -> Result<(), AuthFlowError> {
        self.require_screen("set_email", AuthScreen::SignIn)?;
        let result = set_bounded(
            &mut self.email,
            value.into(),
            AuthInputField::Email,
            EMAIL_MAXIMUM,
        );
        if result.is_ok() {
            self.invalidate_pending_request();
        }
        result
    }

    pub fn set_second_factor_input(&mut self, value: impl AsRef<str>) -> Result<(), AuthFlowError> {
        self.require_screen("set_second_factor_input", AuthScreen::SignIn)?;
        let normalized = normalize_auth_code_input(value.as_ref());
        let result = set_secret_bounded(
            &mut self.second_factor_input,
            normalized,
            AuthInputField::SecondFactor,
            SECOND_FACTOR_MAXIMUM,
        );
        if result.is_ok() {
            self.invalidate_pending_request();
        }
        result
    }

    /// Starts a new auth request and invalidates any response for an older one.
    pub fn submit_auth(&mut self) -> Result<AuthFlowEffect, AuthFlowError> {
        self.require_screen("submit_auth", AuthScreen::SignIn)?;
        require_nonempty(&self.username, AuthInputField::Username)?;
        require_nonempty(self.password.as_str(), AuthInputField::Password)?;

        let username = self.username.trim().to_string();
        match self.mode {
            AuthMode::Login => {
                let request_id = self.allocate_request_id()?;
                self.pending_request = Some(PendingAuthRequest::Login(request_id));
                Ok(AuthFlowEffect::Login {
                    request_id,
                    username,
                    password: self.password.clone(),
                    second_factor: if self.two_factor_visible {
                        classify_auth_code(self.second_factor_input.as_str())
                    } else {
                        AuthSecondFactor {
                            code: AuthSecret::default(),
                            recovery_code: AuthSecret::default(),
                        }
                    },
                })
            }
            AuthMode::Register => {
                require_nonempty(&self.email, AuthInputField::Email)?;
                let email = self.email.trim().to_string();
                if !valid_signup_email(&email) {
                    return Err(AuthFlowError::InvalidSignupEmail);
                }
                let request_id = self.allocate_request_id()?;
                self.pending_request = Some(PendingAuthRequest::Register(request_id));
                Ok(AuthFlowEffect::Register {
                    request_id,
                    username,
                    password: self.password.clone(),
                    email,
                })
            }
        }
    }

    /// Applies a completion only when it belongs to the currently pending auth
    /// operation and has the matching operation kind.
    pub fn complete_auth(
        &mut self,
        request_id: AuthRequestId,
        completion: AuthCompletion,
    ) -> Option<AuthFlowEffect> {
        let accepted = match completion {
            AuthCompletion::PasswordResetSucceeded => matches!(
                self.pending_request,
                Some(PendingAuthRequest::ResetPassword(current)) if current == request_id
            ),
            AuthCompletion::Authenticated => matches!(
                self.pending_request,
                Some(PendingAuthRequest::Login(current) | PendingAuthRequest::Register(current))
                    if current == request_id
            ),
            AuthCompletion::TwoFactorRequired => matches!(
                self.pending_request,
                Some(PendingAuthRequest::Login(current)) if current == request_id
            ),
        };
        if !accepted {
            return None;
        }

        match completion {
            AuthCompletion::Authenticated => {
                self.pending_request = None;
                self.clear_login_credentials();
                Some(AuthFlowEffect::NavigateToRealmDirectory)
            }
            AuthCompletion::TwoFactorRequired => {
                self.pending_request = None;
                self.two_factor_visible = true;
                self.status = AuthStatus::TwoFactorRequired;
                None
            }
            AuthCompletion::PasswordResetSucceeded => {
                self.pending_request = None;
                self.clear_reset_password();
                self.screen = AuthScreen::SignIn;
                self.status = AuthStatus::Idle;
                None
            }
        }
    }

    pub fn open_password_reset_request(&mut self) {
        self.invalidate_pending_request();
        self.clear_login_credentials();
        self.clear_reset_password();
        self.screen = AuthScreen::PasswordResetRequest;
        self.forgot_username.clear();
        self.password_reset_request_status = PasswordResetRequestStatus::Idle;
    }

    pub fn set_forgot_username(&mut self, value: impl Into<String>) -> Result<(), AuthFlowError> {
        self.require_screen("set_forgot_username", AuthScreen::PasswordResetRequest)?;
        let result = set_bounded(
            &mut self.forgot_username,
            value.into(),
            AuthInputField::ForgotUsername,
            USERNAME_MAXIMUM,
        );
        if result.is_ok() {
            self.invalidate_pending_request();
            self.password_reset_request_status = PasswordResetRequestStatus::Idle;
        }
        result
    }

    pub fn submit_password_reset_request(&mut self) -> Result<AuthFlowEffect, AuthFlowError> {
        self.require_screen(
            "submit_password_reset_request",
            AuthScreen::PasswordResetRequest,
        )?;
        require_nonempty(&self.forgot_username, AuthInputField::ForgotUsername)?;
        let request_id = self.allocate_request_id()?;
        self.pending_request = Some(PendingAuthRequest::PasswordResetRequest(request_id));
        self.password_reset_request_status = PasswordResetRequestStatus::Idle;
        Ok(AuthFlowEffect::RequestPasswordReset {
            request_id,
            username: self.forgot_username.trim().to_string(),
        })
    }

    pub fn complete_password_reset_request(
        &mut self,
        request_id: AuthRequestId,
        outcome: PasswordResetRequestOutcome,
    ) -> bool {
        if !matches!(
            self.pending_request,
            Some(PendingAuthRequest::PasswordResetRequest(current)) if current == request_id
        ) {
            return false;
        }
        self.pending_request = None;
        self.password_reset_request_status = match outcome {
            PasswordResetRequestOutcome::Sent | PasswordResetRequestOutcome::OpaqueFailure => {
                PasswordResetRequestStatus::Sent
            }
            PasswordResetRequestOutcome::RateLimited => PasswordResetRequestStatus::RateLimited,
        };
        true
    }

    pub fn open_reset_password(&mut self, token: impl Into<String>) -> Result<(), AuthFlowError> {
        let token = token.into();
        if token.trim().is_empty() {
            return Err(AuthFlowError::MissingResetToken);
        }
        if token.encode_utf16().count() > RESET_TOKEN_MAXIMUM
            || token
                .chars()
                .any(|character| character.is_whitespace() || character.is_control())
        {
            return Err(AuthFlowError::InvalidResetToken);
        }
        self.clear_login_credentials();
        self.invalidate_pending_request();
        self.screen = AuthScreen::ResetPassword;
        self.reset_token = AuthSecret::from(token);
        self.new_password.clear();
        self.password_confirmation.clear();
        Ok(())
    }

    pub fn set_new_password(&mut self, value: impl Into<String>) -> Result<(), AuthFlowError> {
        self.require_screen("set_new_password", AuthScreen::ResetPassword)?;
        let result = set_secret_bounded(
            &mut self.new_password,
            value.into(),
            AuthInputField::NewPassword,
            PASSWORD_MAXIMUM,
        );
        if result.is_ok() {
            self.invalidate_pending_request();
        }
        result
    }

    pub fn set_password_confirmation(
        &mut self,
        value: impl Into<String>,
    ) -> Result<(), AuthFlowError> {
        self.require_screen("set_password_confirmation", AuthScreen::ResetPassword)?;
        let result = set_secret_bounded(
            &mut self.password_confirmation,
            value.into(),
            AuthInputField::PasswordConfirmation,
            PASSWORD_MAXIMUM,
        );
        if result.is_ok() {
            self.invalidate_pending_request();
        }
        result
    }

    pub fn submit_reset_password(&mut self) -> Result<AuthFlowEffect, AuthFlowError> {
        self.require_screen("submit_reset_password", AuthScreen::ResetPassword)?;
        require_nonempty(self.new_password.as_str(), AuthInputField::NewPassword)?;
        require_nonempty(
            self.password_confirmation.as_str(),
            AuthInputField::PasswordConfirmation,
        )?;
        if self.new_password.as_str() != self.password_confirmation.as_str() {
            return Err(AuthFlowError::PasswordConfirmationMismatch);
        }
        let request_id = self.allocate_request_id()?;
        self.pending_request = Some(PendingAuthRequest::ResetPassword(request_id));
        Ok(AuthFlowEffect::ResetPassword {
            request_id,
            token: self.reset_token.clone(),
            password: self.new_password.clone(),
        })
    }

    pub fn back(&mut self) -> Result<Option<AuthFlowEffect>, AuthFlowError> {
        match self.screen {
            AuthScreen::SignIn => {
                self.invalidate_pending_request();
                self.clear_login_credentials();
                Ok(Some(AuthFlowEffect::NavigateToModeSelection))
            }
            AuthScreen::PasswordResetRequest => {
                self.invalidate_pending_request();
                self.screen = AuthScreen::SignIn;
                Ok(None)
            }
            AuthScreen::ResetPassword => {
                self.invalidate_pending_request();
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

    fn allocate_request_id(&mut self) -> Result<AuthRequestId, AuthFlowError> {
        let request_id = AuthRequestId(self.next_request_id);
        self.next_request_id = self
            .next_request_id
            .checked_add(1)
            .ok_or(AuthFlowError::RequestIdExhausted)?;
        Ok(request_id)
    }

    fn invalidate_pending_request(&mut self) {
        self.pending_request = None;
    }

    fn clear_two_factor_challenge(&mut self) {
        self.second_factor_input.clear();
        self.two_factor_visible = false;
        self.status = AuthStatus::Idle;
    }

    // A reset transition invalidates every credential tied to the sign-in challenge.
    fn clear_login_credentials(&mut self) {
        self.password.clear();
        self.clear_two_factor_challenge();
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
            code: AuthSecret::from(compact),
            recovery_code: AuthSecret::default(),
        }
    } else {
        AuthSecondFactor {
            code: AuthSecret::default(),
            recovery_code: AuthSecret::from(trimmed.to_string()),
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

fn set_secret_bounded(
    destination: &mut AuthSecret,
    value: String,
    field: AuthInputField,
    maximum: usize,
) -> Result<(), AuthFlowError> {
    if value.encode_utf16().count() > maximum {
        return Err(AuthFlowError::InputTooLong { field, maximum });
    }
    *destination = AuthSecret::from(value);
    Ok(())
}

fn require_nonempty(value: &str, field: AuthInputField) -> Result<(), AuthFlowError> {
    if value.trim().is_empty() {
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
