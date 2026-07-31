use super::{
    AuthScreen, CharacterSortMode, OfflinePlayerClass, OnlineShellError, ServerMode,
    WocShellController, WocShellEffect, WocShellError, WocShellScreen,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShellRoute {
    Auth(AuthRoute),
    Mode(ModeRoute),
    Offline(OfflineRoute),
    Characters(CharacterRoute),
    Realms(RealmRoute),
    Welcome(WelcomeRoute),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthRoute {
    SetUsername,
    SetPassword,
    SetEmail,
    SetSecondFactor,
    Submit,
    Back,
    ToggleMode,
    OpenPasswordResetRequest,
    SetForgotUsername,
    SubmitForgot,
    BackFromForgot,
    SetResetPassword,
    SetResetConfirmation,
    SubmitReset,
    BackFromReset,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModeRoute {
    ToggleMenu,
    Select(ServerMode),
    Play,
    CopyContractAddress,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OfflineRoute {
    Back,
    Submit,
    SetName,
    SelectClass(OfflinePlayerClass),
    SelectSkin(u16),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharacterRoute {
    ChangeRealm,
    ToggleSort,
    SetSort(CharacterSortMode),
    Back,
    OpenCreate,
    Primary,
    CancelTakeOver,
    ConfirmTakeOver,
    SetDeleteConfirmation,
    CancelDelete,
    SubmitDelete,
    Create(CharacterCreateRoute),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharacterCreateRoute {
    SetName,
    SelectClass(OfflinePlayerClass),
    SelectSkin(u16),
    Back,
    Submit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RealmRoute {
    Back,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WelcomeRoute {
    Continue,
    JoinDiscord,
    OpenArmory,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShellRouteError {
    UnknownRoute(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShellHostEffect {
    CopyContractAddress,
    ToggleCharacterSortMenu,
    JoinDiscord,
    OpenArmory,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShellRouteEffect {
    Woc(WocShellEffect),
    Host(ShellHostEffect),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShellRouteDispatchError {
    Parse(ShellRouteError),
    MissingTextValue {
        route: ShellRoute,
    },
    InvalidScreen {
        route: ShellRoute,
        actual: WocShellScreen,
    },
    Shell(WocShellError),
}

impl From<ShellRouteError> for ShellRouteDispatchError {
    fn from(error: ShellRouteError) -> Self {
        Self::Parse(error)
    }
}

impl From<WocShellError> for ShellRouteDispatchError {
    fn from(error: WocShellError) -> Self {
        Self::Shell(error)
    }
}

/// Parses only static routes declared in the retained shell views.
///
/// Dynamic retained rows carry host-owned identifiers and intentionally stay outside this parser.
pub fn parse_shell_route(route: &str) -> Result<ShellRoute, ShellRouteError> {
    let parsed = parse_static_shell_route(route);
    parsed.ok_or_else(|| ShellRouteError::UnknownRoute(route.to_owned()))
}

impl WocShellController {
    /// Dispatches a static retained route without granting the UI authority over host capabilities.
    pub fn dispatch_shell_route(
        &mut self,
        route: &str,
        text_value: Option<&str>,
    ) -> Result<Option<ShellRouteEffect>, ShellRouteDispatchError> {
        self.dispatch_shell_intent(parse_shell_route(route)?, text_value)
    }

    pub fn dispatch_shell_intent(
        &mut self,
        route: ShellRoute,
        text_value: Option<&str>,
    ) -> Result<Option<ShellRouteEffect>, ShellRouteDispatchError> {
        match route {
            ShellRoute::Auth(route) => self.dispatch_auth_route(route, text_value),
            ShellRoute::Mode(route) => self.dispatch_mode_route(route),
            ShellRoute::Offline(route) => self.dispatch_offline_route(route, text_value),
            ShellRoute::Characters(route) => self.dispatch_character_route(route, text_value),
            ShellRoute::Realms(RealmRoute::Back) => {
                self.require_route_screen(
                    ShellRoute::Realms(RealmRoute::Back),
                    WocShellScreen::RealmDirectory,
                )?;
                Ok(self.back_from_realms()?.map(ShellRouteEffect::Woc))
            }
            ShellRoute::Welcome(route) => self.dispatch_welcome_route(route),
        }
    }

    fn dispatch_auth_route(
        &mut self,
        route: AuthRoute,
        text_value: Option<&str>,
    ) -> Result<Option<ShellRouteEffect>, ShellRouteDispatchError> {
        let shell_route = ShellRoute::Auth(route);
        self.require_route_screen(shell_route, WocShellScreen::Authentication)?;
        match route {
            AuthRoute::SetUsername => self
                .online_mut()
                .auth_mut()
                .set_username(required_text(shell_route, text_value)?)
                .map_err(auth_error)?,
            AuthRoute::SetPassword => self
                .online_mut()
                .auth_mut()
                .set_password(required_text(shell_route, text_value)?)
                .map_err(auth_error)?,
            AuthRoute::SetEmail => self
                .online_mut()
                .auth_mut()
                .set_email(required_text(shell_route, text_value)?)
                .map_err(auth_error)?,
            AuthRoute::SetSecondFactor => self
                .online_mut()
                .auth_mut()
                .set_second_factor_input(required_text(shell_route, text_value)?)
                .map_err(auth_error)?,
            AuthRoute::Submit => {
                let effect = self.online_mut().submit_auth().map_err(online_error)?;
                return Ok(Some(ShellRouteEffect::Woc(WocShellEffect::Online(effect))));
            }
            AuthRoute::Back => {
                self.require_auth_screen("back_from_sign_in", AuthScreen::SignIn)?;
                self.back_from_authentication()?;
            }
            AuthRoute::ToggleMode => {
                self.require_auth_screen("toggle_auth_mode", AuthScreen::SignIn)?;
                self.online_mut().auth_mut().toggle_auth_mode();
            }
            AuthRoute::OpenPasswordResetRequest => {
                self.require_auth_screen("open_password_reset_request", AuthScreen::SignIn)?;
                self.online_mut().auth_mut().open_password_reset_request();
            }
            AuthRoute::SetForgotUsername => self
                .online_mut()
                .auth_mut()
                .set_forgot_username(required_text(shell_route, text_value)?)
                .map_err(auth_error)?,
            AuthRoute::SubmitForgot => {
                let effect = self
                    .online_mut()
                    .auth_mut()
                    .submit_password_reset_request()
                    .map_err(auth_error)?;
                return Ok(Some(ShellRouteEffect::Woc(WocShellEffect::Online(
                    super::OnlineShellEffect::Authentication(effect),
                ))));
            }
            AuthRoute::BackFromForgot => {
                self.require_auth_screen(
                    "back_from_password_reset_request",
                    AuthScreen::PasswordResetRequest,
                )?;
                self.online_mut().auth_mut().back().map_err(auth_error)?;
            }
            AuthRoute::BackFromReset => {
                self.require_auth_screen("back_from_reset_password", AuthScreen::ResetPassword)?;
                self.online_mut().auth_mut().back().map_err(auth_error)?;
            }
            AuthRoute::SetResetPassword => self
                .online_mut()
                .auth_mut()
                .set_new_password(required_text(shell_route, text_value)?)
                .map_err(auth_error)?,
            AuthRoute::SetResetConfirmation => self
                .online_mut()
                .auth_mut()
                .set_password_confirmation(required_text(shell_route, text_value)?)
                .map_err(auth_error)?,
            AuthRoute::SubmitReset => {
                let effect = self
                    .online_mut()
                    .auth_mut()
                    .submit_reset_password()
                    .map_err(auth_error)?;
                return Ok(Some(ShellRouteEffect::Woc(WocShellEffect::Online(
                    super::OnlineShellEffect::Authentication(effect),
                ))));
            }
        }
        Ok(None)
    }

    fn dispatch_mode_route(
        &mut self,
        route: ModeRoute,
    ) -> Result<Option<ShellRouteEffect>, ShellRouteDispatchError> {
        let shell_route = ShellRoute::Mode(route);
        self.require_route_screen(shell_route, WocShellScreen::ModeSelection)?;
        match route {
            ModeRoute::ToggleMenu => self.toggle_mode_menu()?,
            ModeRoute::Select(mode) => self.select_mode(mode)?,
            ModeRoute::Play => return Ok(self.play()?.map(ShellRouteEffect::Woc)),
            ModeRoute::CopyContractAddress => {
                return Ok(Some(ShellRouteEffect::Host(
                    ShellHostEffect::CopyContractAddress,
                )));
            }
        }
        Ok(None)
    }

    fn dispatch_offline_route(
        &mut self,
        route: OfflineRoute,
        text_value: Option<&str>,
    ) -> Result<Option<ShellRouteEffect>, ShellRouteDispatchError> {
        let shell_route = ShellRoute::Offline(route);
        self.require_route_screen(shell_route, WocShellScreen::OfflinePicker)?;
        match route {
            OfflineRoute::Back => self.back_from_offline_picker()?,
            OfflineRoute::Submit => {
                return Ok(Some(ShellRouteEffect::Woc(self.submit_offline_picker()?)));
            }
            OfflineRoute::SetName => {
                self.set_offline_name(required_text(shell_route, text_value)?)?
            }
            OfflineRoute::SelectClass(player_class) => self.set_offline_class(player_class)?,
            OfflineRoute::SelectSkin(skin_variant) => self.set_offline_skin(skin_variant)?,
        }
        Ok(None)
    }

    fn dispatch_character_route(
        &mut self,
        route: CharacterRoute,
        text_value: Option<&str>,
    ) -> Result<Option<ShellRouteEffect>, ShellRouteDispatchError> {
        let shell_route = ShellRoute::Characters(route);
        match route {
            CharacterRoute::ChangeRealm => {
                self.require_route_screen(shell_route, WocShellScreen::CharacterSelection)?;
                self.online_mut().change_realm().map_err(online_error)?;
            }
            CharacterRoute::ToggleSort => {
                self.require_route_screen(shell_route, WocShellScreen::CharacterSelection)?;
                return Ok(Some(ShellRouteEffect::Host(
                    ShellHostEffect::ToggleCharacterSortMenu,
                )));
            }
            CharacterRoute::SetSort(mode) => {
                self.require_route_screen(shell_route, WocShellScreen::CharacterSelection)?;
                let effect = self
                    .online_mut()
                    .set_character_sort_mode(mode)
                    .map_err(online_error)?;
                return Ok(Some(ShellRouteEffect::Woc(WocShellEffect::Online(effect))));
            }
            CharacterRoute::Back => {
                return Ok(self.back_from_characters()?.map(ShellRouteEffect::Woc))
            }
            CharacterRoute::OpenCreate => {
                self.require_route_screen(shell_route, WocShellScreen::CharacterSelection)?;
                self.online_mut()
                    .open_character_create()
                    .map_err(online_error)?;
            }
            CharacterRoute::Primary => {
                self.require_route_screen(shell_route, WocShellScreen::CharacterSelection)?;
                let effect = self
                    .online_mut()
                    .character_primary_action()
                    .map_err(online_error)?;
                return Ok(Some(ShellRouteEffect::Woc(WocShellEffect::Online(effect))));
            }
            CharacterRoute::CancelTakeOver => {
                self.require_route_screen(shell_route, WocShellScreen::CharacterSelection)?;
                self.online_mut()
                    .cancel_character_takeover()
                    .map_err(online_error)?;
            }
            CharacterRoute::ConfirmTakeOver => {
                self.require_route_screen(shell_route, WocShellScreen::CharacterSelection)?;
                let effect = self
                    .online_mut()
                    .confirm_character_takeover()
                    .map_err(online_error)?;
                return Ok(Some(ShellRouteEffect::Woc(WocShellEffect::Online(effect))));
            }
            CharacterRoute::SetDeleteConfirmation => {
                self.require_route_screen(shell_route, WocShellScreen::CharacterSelection)?;
                self.online_mut()
                    .set_character_delete_confirmation(required_text(shell_route, text_value)?)
                    .map_err(online_error)?;
            }
            CharacterRoute::CancelDelete => {
                self.require_route_screen(shell_route, WocShellScreen::CharacterSelection)?;
                self.online_mut()
                    .cancel_character_delete()
                    .map_err(online_error)?;
            }
            CharacterRoute::SubmitDelete => {
                self.require_route_screen(shell_route, WocShellScreen::CharacterSelection)?;
                let effect = self
                    .online_mut()
                    .submit_character_delete()
                    .map_err(online_error)?;
                return Ok(Some(ShellRouteEffect::Woc(WocShellEffect::Online(effect))));
            }
            CharacterRoute::Create(route) => {
                return self.dispatch_character_create_route(route, text_value);
            }
        }
        Ok(None)
    }

    fn dispatch_character_create_route(
        &mut self,
        route: CharacterCreateRoute,
        text_value: Option<&str>,
    ) -> Result<Option<ShellRouteEffect>, ShellRouteDispatchError> {
        let shell_route = ShellRoute::Characters(CharacterRoute::Create(route));
        self.require_route_screen(shell_route, WocShellScreen::CharacterCreation)?;
        match route {
            CharacterCreateRoute::SetName => self
                .online_mut()
                .set_character_create_name(required_text(shell_route, text_value)?)
                .map_err(online_error)?,
            CharacterCreateRoute::SelectClass(player_class) => self
                .online_mut()
                .set_character_create_class(player_class)
                .map_err(online_error)?,
            CharacterCreateRoute::SelectSkin(skin_variant) => self
                .online_mut()
                .set_character_create_skin(skin_variant)
                .map_err(online_error)?,
            CharacterCreateRoute::Back => {
                return Ok(self.back_from_characters()?.map(ShellRouteEffect::Woc))
            }
            CharacterCreateRoute::Submit => {
                let effect = self
                    .online_mut()
                    .submit_character_create()
                    .map_err(online_error)?;
                return Ok(Some(ShellRouteEffect::Woc(WocShellEffect::Online(effect))));
            }
        }
        Ok(None)
    }

    fn dispatch_welcome_route(
        &mut self,
        route: WelcomeRoute,
    ) -> Result<Option<ShellRouteEffect>, ShellRouteDispatchError> {
        let shell_route = ShellRoute::Welcome(route);
        self.require_route_screen(shell_route, WocShellScreen::Welcome)?;
        match route {
            WelcomeRoute::Continue => Ok(Some(ShellRouteEffect::Woc(
                self.continue_offline_welcome()?,
            ))),
            WelcomeRoute::JoinDiscord => {
                Ok(Some(ShellRouteEffect::Host(ShellHostEffect::JoinDiscord)))
            }
            WelcomeRoute::OpenArmory => {
                Ok(Some(ShellRouteEffect::Host(ShellHostEffect::OpenArmory)))
            }
        }
    }

    fn require_route_screen(
        &self,
        route: ShellRoute,
        expected: WocShellScreen,
    ) -> Result<(), ShellRouteDispatchError> {
        let actual = self.screen();
        if actual == expected {
            Ok(())
        } else {
            Err(ShellRouteDispatchError::InvalidScreen { route, actual })
        }
    }

    fn require_auth_screen(
        &self,
        action: &'static str,
        expected: AuthScreen,
    ) -> Result<(), ShellRouteDispatchError> {
        let actual = self.online().auth().screen();
        if actual == expected {
            Ok(())
        } else {
            Err(auth_error(super::AuthFlowError::InvalidScreen {
                action,
                expected,
                actual,
            }))
        }
    }
}

fn required_text(route: ShellRoute, value: Option<&str>) -> Result<&str, ShellRouteDispatchError> {
    value.ok_or(ShellRouteDispatchError::MissingTextValue { route })
}

fn auth_error(error: super::AuthFlowError) -> ShellRouteDispatchError {
    ShellRouteDispatchError::Shell(WocShellError::Auth(error))
}

fn online_error(error: OnlineShellError) -> ShellRouteDispatchError {
    ShellRouteDispatchError::Shell(WocShellError::Online(error))
}

fn parse_static_shell_route(route: &str) -> Option<ShellRoute> {
    let parsed = match route {
        "woc.shell.auth.set_username" => Some(ShellRoute::Auth(AuthRoute::SetUsername)),
        "woc.shell.auth.set_password" => Some(ShellRoute::Auth(AuthRoute::SetPassword)),
        "woc.shell.auth.set_email" => Some(ShellRoute::Auth(AuthRoute::SetEmail)),
        "woc.shell.auth.set_two_factor" => Some(ShellRoute::Auth(AuthRoute::SetSecondFactor)),
        "woc.shell.auth.submit" => Some(ShellRoute::Auth(AuthRoute::Submit)),
        "woc.shell.auth.back" => Some(ShellRoute::Auth(AuthRoute::Back)),
        "woc.shell.auth.toggle_mode" => Some(ShellRoute::Auth(AuthRoute::ToggleMode)),
        "woc.shell.auth.open_forgot" => Some(ShellRoute::Auth(AuthRoute::OpenPasswordResetRequest)),
        "woc.shell.auth.forgot.set_username" => {
            Some(ShellRoute::Auth(AuthRoute::SetForgotUsername))
        }
        "woc.shell.auth.forgot.submit" => Some(ShellRoute::Auth(AuthRoute::SubmitForgot)),
        "woc.shell.auth.forgot.back" => Some(ShellRoute::Auth(AuthRoute::BackFromForgot)),
        "woc.shell.auth.reset.set_password" => Some(ShellRoute::Auth(AuthRoute::SetResetPassword)),
        "woc.shell.auth.reset.set_confirmation" => {
            Some(ShellRoute::Auth(AuthRoute::SetResetConfirmation))
        }
        "woc.shell.auth.reset.submit" => Some(ShellRoute::Auth(AuthRoute::SubmitReset)),
        "woc.shell.auth.reset.back" => Some(ShellRoute::Auth(AuthRoute::BackFromReset)),
        "woc.shell.mode.toggle_menu" => Some(ShellRoute::Mode(ModeRoute::ToggleMenu)),
        "woc.shell.mode.select.online" => {
            Some(ShellRoute::Mode(ModeRoute::Select(ServerMode::Online)))
        }
        "woc.shell.mode.select.offline" => {
            Some(ShellRoute::Mode(ModeRoute::Select(ServerMode::Offline)))
        }
        "woc.shell.mode.play" => Some(ShellRoute::Mode(ModeRoute::Play)),
        "woc.shell.mode.copy_contract" => Some(ShellRoute::Mode(ModeRoute::CopyContractAddress)),
        "woc.shell.offline.back" => Some(ShellRoute::Offline(OfflineRoute::Back)),
        "woc.shell.offline.enter_world" => Some(ShellRoute::Offline(OfflineRoute::Submit)),
        "woc.shell.offline.set_name" => Some(ShellRoute::Offline(OfflineRoute::SetName)),
        "woc.shell.characters.change_realm" => {
            Some(ShellRoute::Characters(CharacterRoute::ChangeRealm))
        }
        "woc.shell.characters.toggle_sort" => {
            Some(ShellRoute::Characters(CharacterRoute::ToggleSort))
        }
        "woc.shell.characters.sort.level" => Some(ShellRoute::Characters(CharacterRoute::SetSort(
            CharacterSortMode::Level,
        ))),
        "woc.shell.characters.sort.name" => Some(ShellRoute::Characters(CharacterRoute::SetSort(
            CharacterSortMode::Name,
        ))),
        "woc.shell.characters.sort.recent" => Some(ShellRoute::Characters(
            CharacterRoute::SetSort(CharacterSortMode::Recent),
        )),
        "woc.shell.characters.sort.playtime" => Some(ShellRoute::Characters(
            CharacterRoute::SetSort(CharacterSortMode::Playtime),
        )),
        "woc.shell.characters.back" => Some(ShellRoute::Characters(CharacterRoute::Back)),
        "woc.shell.characters.new" => Some(ShellRoute::Characters(CharacterRoute::OpenCreate)),
        "woc.shell.characters.primary" => Some(ShellRoute::Characters(CharacterRoute::Primary)),
        "woc.shell.characters.takeover.cancel" => {
            Some(ShellRoute::Characters(CharacterRoute::CancelTakeOver))
        }
        "woc.shell.characters.takeover.confirm" => {
            Some(ShellRoute::Characters(CharacterRoute::ConfirmTakeOver))
        }
        "woc.shell.characters.delete.set_confirmation" => Some(ShellRoute::Characters(
            CharacterRoute::SetDeleteConfirmation,
        )),
        "woc.shell.characters.delete.cancel" => {
            Some(ShellRoute::Characters(CharacterRoute::CancelDelete))
        }
        "woc.shell.characters.delete.submit" => {
            Some(ShellRoute::Characters(CharacterRoute::SubmitDelete))
        }
        "woc.shell.characters.create.set_name" => Some(ShellRoute::Characters(
            CharacterRoute::Create(CharacterCreateRoute::SetName),
        )),
        "woc.shell.characters.create.back" => Some(ShellRoute::Characters(CharacterRoute::Create(
            CharacterCreateRoute::Back,
        ))),
        "woc.shell.characters.create.submit" => Some(ShellRoute::Characters(
            CharacterRoute::Create(CharacterCreateRoute::Submit),
        )),
        "woc.shell.realms.back" => Some(ShellRoute::Realms(RealmRoute::Back)),
        "woc.shell.welcome.continue" => Some(ShellRoute::Welcome(WelcomeRoute::Continue)),
        "woc.shell.welcome.join_discord" => Some(ShellRoute::Welcome(WelcomeRoute::JoinDiscord)),
        "woc.shell.welcome.open_armory" => Some(ShellRoute::Welcome(WelcomeRoute::OpenArmory)),
        _ => None,
    };
    parsed
        .or_else(|| parse_offline_picker_route(route))
        .or_else(|| parse_character_create_route(route))
}

fn parse_offline_picker_route(route: &str) -> Option<ShellRoute> {
    if let Some(player_class) = route
        .strip_prefix("woc.shell.offline.select_class.")
        .and_then(OfflinePlayerClass::parse)
    {
        return Some(ShellRoute::Offline(OfflineRoute::SelectClass(player_class)));
    }
    parse_skin_variant(route, "woc.shell.offline.select_skin.")
        .map(|skin_variant| ShellRoute::Offline(OfflineRoute::SelectSkin(skin_variant)))
}

fn parse_character_create_route(route: &str) -> Option<ShellRoute> {
    if let Some(player_class) = route
        .strip_prefix("woc.shell.characters.create.select_class.")
        .and_then(OfflinePlayerClass::parse)
    {
        return Some(ShellRoute::Characters(CharacterRoute::Create(
            CharacterCreateRoute::SelectClass(player_class),
        )));
    }
    parse_skin_variant(route, "woc.shell.characters.create.select_skin.").map(|skin_variant| {
        ShellRoute::Characters(CharacterRoute::Create(CharacterCreateRoute::SelectSkin(
            skin_variant,
        )))
    })
}

fn parse_skin_variant(route: &str, prefix: &str) -> Option<u16> {
    match route.strip_prefix(prefix)? {
        "0" => Some(0),
        "1" => Some(1),
        "2" => Some(2),
        "3" => Some(3),
        _ => None,
    }
}
