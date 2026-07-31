use crate::preferences::OptionsPanelId;

use super::{ClientGameplayIntent, ClientInputDevice, ClientInputEvent};

pub const DESKTOP_ACTION_SLOT_COUNT: u8 = 23;
pub const MOBILE_ACTIONS_PER_PAGE: u8 = 5;
pub const MOBILE_ACTION_SOURCE_SLOT_START: u8 = 1;
pub const MOBILE_ACTION_SOURCE_SLOT_COUNT: u8 = 10;
pub const MOBILE_ACTION_PAGE_COUNT: u8 = 2;
pub const TOUCH_CONSUMABLE_SLOT_COUNT: u8 = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HudRoute {
    DesktopAction(u8),
    Touch(TouchHudRoute),
    Pause(PauseHudRoute),
    Lockpick(LockpickHudRoute),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TouchHudRoute {
    ActivateAction(u8),
    Attack,
    CycleTarget,
    Interact,
    Jump,
    NextActionPage,
    UseConsumable(u8),
    OpenChat,
    OpenSocial,
    OpenQuests,
    OpenMore,
    ToggleConsumables,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PauseHudRoute {
    Open,
    OpenOptions(OptionsPanelId),
    Logout,
    ReturnToGame,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LockpickHudAction {
    HardSet,
    Set,
    Steady,
    Ease,
    Drop,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LockpickHudRoute {
    Engage { ante: u8 },
    Action(LockpickHudAction),
    Abort,
    CloseSelector,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HudRouteError {
    UnknownRoute(String),
    PauseClosed,
    BugReportUnavailable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HudHostEffect {
    MobileAttack,
    Jump,
    SetMobileActionPage { page: u8 },
    UseConsumable { index: u8 },
    OpenChat,
    OpenSocial,
    OpenQuestLog,
    OpenMore,
    SetConsumablesVisible { visible: bool },
    SetPauseVisible { visible: bool },
    OpenOptions(OptionsPanelId),
    RequestLogout,
    RequestLockpickEngage { ante: u8 },
    RequestLockpickAction { action: LockpickHudAction },
    RequestLockpickAbort,
    CloseLockpickSelector,
}

#[derive(Clone, Debug, PartialEq)]
pub enum HudRouteEffect {
    Input(ClientInputEvent),
    Host(HudHostEffect),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HudRouteController {
    mobile_action_page: u8,
    consumables_visible: bool,
    pause_visible: bool,
}

impl HudRouteController {
    pub fn mobile_action_page(&self) -> u8 {
        self.mobile_action_page
    }

    pub fn consumables_visible(&self) -> bool {
        self.consumables_visible
    }

    pub fn pause_visible(&self) -> bool {
        self.pause_visible
    }

    pub fn dispatch_route(
        &mut self,
        route: &str,
        online: bool,
    ) -> Result<HudRouteEffect, HudRouteError> {
        self.dispatch(parse_hud_route(route)?, online)
    }

    pub fn dispatch(
        &mut self,
        route: HudRoute,
        online: bool,
    ) -> Result<HudRouteEffect, HudRouteError> {
        match route {
            HudRoute::DesktopAction(slot) => Ok(input(
                ClientInputDevice::KeyboardMouse,
                ClientGameplayIntent::CastSlot {
                    slot: i32::from(slot),
                },
            )),
            HudRoute::Touch(route) => Ok(self.dispatch_touch(route)),
            HudRoute::Pause(route) => self.dispatch_pause(route, online),
            HudRoute::Lockpick(route) => Ok(self.dispatch_lockpick(route)),
        }
    }

    fn dispatch_touch(&mut self, route: TouchHudRoute) -> HudRouteEffect {
        match route {
            TouchHudRoute::ActivateAction(button) => {
                let slot = MOBILE_ACTION_SOURCE_SLOT_START
                    + self.mobile_action_page * MOBILE_ACTIONS_PER_PAGE
                    + button;
                input(
                    ClientInputDevice::Touch,
                    ClientGameplayIntent::CastSlot {
                        slot: i32::from(slot),
                    },
                )
            }
            TouchHudRoute::Attack => host(HudHostEffect::MobileAttack),
            TouchHudRoute::CycleTarget => input(
                ClientInputDevice::Touch,
                ClientGameplayIntent::CycleTarget { friendly: false },
            ),
            TouchHudRoute::Interact => {
                input(ClientInputDevice::Touch, ClientGameplayIntent::Interact)
            }
            TouchHudRoute::Jump => host(HudHostEffect::Jump),
            TouchHudRoute::NextActionPage => {
                self.mobile_action_page = (self.mobile_action_page + 1) % MOBILE_ACTION_PAGE_COUNT;
                host(HudHostEffect::SetMobileActionPage {
                    page: self.mobile_action_page,
                })
            }
            TouchHudRoute::UseConsumable(index) => host(HudHostEffect::UseConsumable { index }),
            TouchHudRoute::OpenChat => host(HudHostEffect::OpenChat),
            TouchHudRoute::OpenSocial => host(HudHostEffect::OpenSocial),
            TouchHudRoute::OpenQuests => host(HudHostEffect::OpenQuestLog),
            TouchHudRoute::OpenMore => host(HudHostEffect::OpenMore),
            TouchHudRoute::ToggleConsumables => {
                self.consumables_visible = !self.consumables_visible;
                host(HudHostEffect::SetConsumablesVisible {
                    visible: self.consumables_visible,
                })
            }
        }
    }

    fn dispatch_pause(
        &mut self,
        route: PauseHudRoute,
        online: bool,
    ) -> Result<HudRouteEffect, HudRouteError> {
        match route {
            PauseHudRoute::Open => {
                self.pause_visible = true;
                Ok(host(HudHostEffect::SetPauseVisible { visible: true }))
            }
            PauseHudRoute::OpenOptions(panel) => {
                self.require_pause_visible()?;
                if panel == OptionsPanelId::BugReport && !online {
                    return Err(HudRouteError::BugReportUnavailable);
                }
                Ok(host(HudHostEffect::OpenOptions(panel)))
            }
            PauseHudRoute::Logout => {
                self.require_pause_visible()?;
                Ok(host(HudHostEffect::RequestLogout))
            }
            PauseHudRoute::ReturnToGame => {
                self.require_pause_visible()?;
                self.pause_visible = false;
                Ok(host(HudHostEffect::SetPauseVisible { visible: false }))
            }
        }
    }

    fn dispatch_lockpick(&mut self, route: LockpickHudRoute) -> HudRouteEffect {
        match route {
            LockpickHudRoute::Engage { ante } => {
                host(HudHostEffect::RequestLockpickEngage { ante })
            }
            LockpickHudRoute::Action(action) => {
                host(HudHostEffect::RequestLockpickAction { action })
            }
            LockpickHudRoute::Abort => host(HudHostEffect::RequestLockpickAbort),
            LockpickHudRoute::CloseSelector => host(HudHostEffect::CloseLockpickSelector),
        }
    }

    fn require_pause_visible(&self) -> Result<(), HudRouteError> {
        if self.pause_visible {
            Ok(())
        } else {
            Err(HudRouteError::PauseClosed)
        }
    }
}

pub fn parse_hud_route(route: &str) -> Result<HudRoute, HudRouteError> {
    parse_static_hud_route(route).ok_or_else(|| HudRouteError::UnknownRoute(route.to_owned()))
}

fn parse_static_hud_route(route: &str) -> Option<HudRoute> {
    if let Some(slot) =
        parse_numbered_route(route, "woc.hud.action.activate.", DESKTOP_ACTION_SLOT_COUNT)
    {
        return Some(HudRoute::DesktopAction(slot));
    }
    if let Some(button) =
        parse_numbered_route(route, "woc.hud.touch.activate.", MOBILE_ACTIONS_PER_PAGE)
    {
        return Some(HudRoute::Touch(TouchHudRoute::ActivateAction(button)));
    }
    if let Some(index) =
        parse_numbered_route(route, "woc.hud.touch.consume.", TOUCH_CONSUMABLE_SLOT_COUNT)
    {
        return Some(HudRoute::Touch(TouchHudRoute::UseConsumable(index)));
    }
    if let Some(route) = parse_lockpick_route(route) {
        return Some(HudRoute::Lockpick(route));
    }
    let route = match route {
        "woc.hud.touch.attack" => HudRoute::Touch(TouchHudRoute::Attack),
        "woc.hud.touch.target_cycle" => HudRoute::Touch(TouchHudRoute::CycleTarget),
        "woc.hud.touch.interact" => HudRoute::Touch(TouchHudRoute::Interact),
        "woc.hud.touch.jump" => HudRoute::Touch(TouchHudRoute::Jump),
        "woc.hud.touch.next_page" => HudRoute::Touch(TouchHudRoute::NextActionPage),
        "woc.hud.touch.open_chat" => HudRoute::Touch(TouchHudRoute::OpenChat),
        "woc.hud.touch.open_social" => HudRoute::Touch(TouchHudRoute::OpenSocial),
        "woc.hud.touch.open_quests" => HudRoute::Touch(TouchHudRoute::OpenQuests),
        "woc.hud.touch.open_more" => HudRoute::Touch(TouchHudRoute::OpenMore),
        "woc.hud.touch.toggle_consumables" => HudRoute::Touch(TouchHudRoute::ToggleConsumables),
        "woc.hud.pause.open" => HudRoute::Pause(PauseHudRoute::Open),
        "woc.hud.pause.open.keybinds" => {
            HudRoute::Pause(PauseHudRoute::OpenOptions(OptionsPanelId::Keybinds))
        }
        "woc.hud.pause.open.controller" => {
            HudRoute::Pause(PauseHudRoute::OpenOptions(OptionsPanelId::Controller))
        }
        "woc.hud.pause.open.graphics" => {
            HudRoute::Pause(PauseHudRoute::OpenOptions(OptionsPanelId::Graphics))
        }
        "woc.hud.pause.open.interface" => {
            HudRoute::Pause(PauseHudRoute::OpenOptions(OptionsPanelId::Interface))
        }
        "woc.hud.pause.open.audio" => {
            HudRoute::Pause(PauseHudRoute::OpenOptions(OptionsPanelId::Audio))
        }
        "woc.hud.pause.open.performance" => {
            HudRoute::Pause(PauseHudRoute::OpenOptions(OptionsPanelId::Performance))
        }
        "woc.hud.pause.open.bug_report" => {
            HudRoute::Pause(PauseHudRoute::OpenOptions(OptionsPanelId::BugReport))
        }
        "woc.hud.pause.logout" => HudRoute::Pause(PauseHudRoute::Logout),
        "woc.hud.pause.return_to_game" => HudRoute::Pause(PauseHudRoute::ReturnToGame),
        _ => return None,
    };
    Some(route)
}

fn parse_lockpick_route(route: &str) -> Option<LockpickHudRoute> {
    if let Some(ante) = route.strip_prefix("woc.hud.lockpick.engage.") {
        return match ante {
            "1" => Some(LockpickHudRoute::Engage { ante: 1 }),
            "2" => Some(LockpickHudRoute::Engage { ante: 2 }),
            "3" => Some(LockpickHudRoute::Engage { ante: 3 }),
            _ => None,
        };
    }
    let route = match route {
        "woc.hud.lockpick.action.hard_set" => LockpickHudRoute::Action(LockpickHudAction::HardSet),
        "woc.hud.lockpick.action.set" => LockpickHudRoute::Action(LockpickHudAction::Set),
        "woc.hud.lockpick.action.steady" => LockpickHudRoute::Action(LockpickHudAction::Steady),
        "woc.hud.lockpick.action.ease" => LockpickHudRoute::Action(LockpickHudAction::Ease),
        "woc.hud.lockpick.action.drop" => LockpickHudRoute::Action(LockpickHudAction::Drop),
        "woc.hud.lockpick.abort" => LockpickHudRoute::Abort,
        "woc.hud.lockpick.close" => LockpickHudRoute::CloseSelector,
        _ => return None,
    };
    Some(route)
}

fn parse_numbered_route(route: &str, prefix: &str, count: u8) -> Option<u8> {
    let value = route.strip_prefix(prefix)?.parse::<u8>().ok()?;
    (value < count).then_some(value)
}

fn input(device: ClientInputDevice, intent: ClientGameplayIntent) -> HudRouteEffect {
    HudRouteEffect::Input(ClientInputEvent { device, intent })
}

fn host(effect: HudHostEffect) -> HudRouteEffect {
    HudRouteEffect::Host(effect)
}
