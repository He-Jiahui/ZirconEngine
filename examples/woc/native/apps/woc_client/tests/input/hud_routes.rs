use woc_client::{
    parse_hud_route, ClientGameplayIntent, ClientInputDevice, ClientInputEvent, HudHostEffect,
    HudRouteController, HudRouteEffect, HudRouteError, LockpickHudAction, MOBILE_ACTIONS_PER_PAGE,
    MOBILE_ACTION_PAGE_COUNT, MOBILE_ACTION_SOURCE_SLOT_COUNT,
};

#[test]
fn every_static_hud_route_in_the_retained_view_parses() {
    for slot in 0..23 {
        assert!(parse_hud_route(&format!("woc.hud.action.activate.{slot}")).is_ok());
    }
    for slot in 0..MOBILE_ACTIONS_PER_PAGE {
        assert!(parse_hud_route(&format!("woc.hud.touch.activate.{slot}")).is_ok());
    }
    for slot in 0..6 {
        assert!(parse_hud_route(&format!("woc.hud.touch.consume.{slot}")).is_ok());
    }
    for route in [
        "woc.hud.pause.open",
        "woc.hud.pause.open.keybinds",
        "woc.hud.pause.open.controller",
        "woc.hud.pause.open.graphics",
        "woc.hud.pause.open.interface",
        "woc.hud.pause.open.audio",
        "woc.hud.pause.open.performance",
        "woc.hud.pause.open.bug_report",
        "woc.hud.pause.logout",
        "woc.hud.pause.return_to_game",
        "woc.hud.touch.attack",
        "woc.hud.touch.target_cycle",
        "woc.hud.touch.interact",
        "woc.hud.touch.jump",
        "woc.hud.touch.next_page",
        "woc.hud.touch.open_chat",
        "woc.hud.touch.open_social",
        "woc.hud.touch.open_quests",
        "woc.hud.touch.open_more",
        "woc.hud.touch.toggle_consumables",
        "woc.hud.lockpick.engage.1",
        "woc.hud.lockpick.engage.2",
        "woc.hud.lockpick.engage.3",
        "woc.hud.lockpick.action.hard_set",
        "woc.hud.lockpick.action.set",
        "woc.hud.lockpick.action.steady",
        "woc.hud.lockpick.action.ease",
        "woc.hud.lockpick.action.drop",
        "woc.hud.lockpick.abort",
        "woc.hud.lockpick.close",
    ] {
        assert!(parse_hud_route(route).is_ok(), "must parse {route}");
    }
}

#[test]
fn lockpick_ante_and_actions_stay_host_requests_until_the_authoritative_descriptor_exists() {
    let mut controller = HudRouteController::default();

    assert_eq!(
        controller.dispatch_route("woc.hud.lockpick.engage.1", true),
        Ok(HudRouteEffect::Host(HudHostEffect::RequestLockpickEngage {
            ante: 1,
        }))
    );
    for (route, action) in [
        (
            "woc.hud.lockpick.action.hard_set",
            LockpickHudAction::HardSet,
        ),
        ("woc.hud.lockpick.action.set", LockpickHudAction::Set),
        ("woc.hud.lockpick.action.steady", LockpickHudAction::Steady),
        ("woc.hud.lockpick.action.ease", LockpickHudAction::Ease),
        ("woc.hud.lockpick.action.drop", LockpickHudAction::Drop),
    ] {
        assert_eq!(
            controller.dispatch_route(route, true),
            Ok(HudRouteEffect::Host(HudHostEffect::RequestLockpickAction {
                action,
            }))
        );
    }
}

#[test]
fn lockpick_abort_and_selector_close_are_distinct_host_effects() {
    let mut controller = HudRouteController::default();

    assert_eq!(
        controller.dispatch_route("woc.hud.lockpick.abort", false),
        Ok(HudRouteEffect::Host(HudHostEffect::RequestLockpickAbort))
    );
    assert_eq!(
        controller.dispatch_route("woc.hud.lockpick.close", false),
        Ok(HudRouteEffect::Host(HudHostEffect::CloseLockpickSelector))
    );
}

#[test]
fn desktop_actions_remain_keyboard_mouse_authority_inputs() {
    let mut controller = HudRouteController::default();

    for slot in 0..23 {
        assert_eq!(
            controller.dispatch_route(&format!("woc.hud.action.activate.{slot}"), true),
            Ok(HudRouteEffect::Input(ClientInputEvent {
                device: ClientInputDevice::KeyboardMouse,
                intent: ClientGameplayIntent::CastSlot { slot },
            }))
        );
    }
}

#[test]
fn mobile_ring_has_two_pages_of_five_source_slots_and_wraps() {
    let mut controller = HudRouteController::default();

    assert_eq!(MOBILE_ACTIONS_PER_PAGE, 5);
    assert_eq!(MOBILE_ACTION_PAGE_COUNT, 2);
    assert_eq!(MOBILE_ACTION_SOURCE_SLOT_COUNT, 10);
    assert_eq!(controller.mobile_action_page(), 0);

    for button in 0..MOBILE_ACTIONS_PER_PAGE {
        assert_eq!(
            controller.dispatch_route(&format!("woc.hud.touch.activate.{button}"), true),
            Ok(HudRouteEffect::Input(ClientInputEvent {
                device: ClientInputDevice::Touch,
                intent: ClientGameplayIntent::CastSlot {
                    slot: i32::from(button) + 1,
                },
            }))
        );
    }

    assert_eq!(
        controller.dispatch_route("woc.hud.touch.next_page", true),
        Ok(HudRouteEffect::Host(HudHostEffect::SetMobileActionPage {
            page: 1,
        }))
    );
    for button in 0..MOBILE_ACTIONS_PER_PAGE {
        assert_eq!(
            controller.dispatch_route(&format!("woc.hud.touch.activate.{button}"), true),
            Ok(HudRouteEffect::Input(ClientInputEvent {
                device: ClientInputDevice::Touch,
                intent: ClientGameplayIntent::CastSlot {
                    slot: i32::from(button) + 6,
                },
            }))
        );
    }
    assert_eq!(
        controller.dispatch_route("woc.hud.touch.next_page", true),
        Ok(HudRouteEffect::Host(HudHostEffect::SetMobileActionPage {
            page: 0,
        }))
    );
}

#[test]
fn hud_routes_connect_interact_but_keep_unimplemented_touch_actions_host_owned() {
    let mut controller = HudRouteController::default();

    assert_eq!(
        controller.dispatch_route("woc.hud.touch.attack", true),
        Ok(HudRouteEffect::Host(HudHostEffect::MobileAttack))
    );
    assert_eq!(
        controller.dispatch_route("woc.hud.touch.interact", true),
        Ok(HudRouteEffect::Input(ClientInputEvent {
            device: ClientInputDevice::Touch,
            intent: ClientGameplayIntent::Interact,
        }))
    );
    assert_eq!(
        controller.dispatch_route("woc.hud.touch.jump", true),
        Ok(HudRouteEffect::Host(HudHostEffect::Jump))
    );
    assert_eq!(
        controller.dispatch_route("woc.hud.touch.consume.5", true),
        Ok(HudRouteEffect::Host(HudHostEffect::UseConsumable {
            index: 5,
        }))
    );
    assert_eq!(
        controller.dispatch_route("woc.hud.touch.target_cycle", true),
        Ok(HudRouteEffect::Input(ClientInputEvent {
            device: ClientInputDevice::Touch,
            intent: ClientGameplayIntent::CycleTarget { friendly: false },
        }))
    );
}

#[test]
fn touch_host_routes_keep_local_visibility_and_surfaces_host_requests() {
    let mut controller = HudRouteController::default();

    assert!(!controller.consumables_visible());
    assert_eq!(
        controller.dispatch_route("woc.hud.touch.toggle_consumables", true),
        Ok(HudRouteEffect::Host(HudHostEffect::SetConsumablesVisible {
            visible: true,
        }))
    );
    assert!(controller.consumables_visible());
    assert_eq!(
        controller.dispatch_route("woc.hud.touch.toggle_consumables", true),
        Ok(HudRouteEffect::Host(HudHostEffect::SetConsumablesVisible {
            visible: false,
        }))
    );
    assert_eq!(
        controller.dispatch_route("woc.hud.touch.open_chat", true),
        Ok(HudRouteEffect::Host(HudHostEffect::OpenChat))
    );
    assert_eq!(
        controller.dispatch_route("woc.hud.touch.open_social", true),
        Ok(HudRouteEffect::Host(HudHostEffect::OpenSocial))
    );
    assert_eq!(
        controller.dispatch_route("woc.hud.touch.open_quests", true),
        Ok(HudRouteEffect::Host(HudHostEffect::OpenQuestLog))
    );
    assert_eq!(
        controller.dispatch_route("woc.hud.touch.open_more", true),
        Ok(HudRouteEffect::Host(HudHostEffect::OpenMore))
    );
}

#[test]
fn pause_routes_gate_options_and_online_bug_reporting() {
    let mut controller = HudRouteController::default();

    assert_eq!(
        controller.dispatch_route("woc.hud.pause.open.graphics", true),
        Err(HudRouteError::PauseClosed)
    );
    assert_eq!(
        controller.dispatch_route("woc.hud.pause.open", false),
        Ok(HudRouteEffect::Host(HudHostEffect::SetPauseVisible {
            visible: true,
        }))
    );
    assert!(controller.pause_visible());
    assert_eq!(
        controller.dispatch_route("woc.hud.pause.open.bug_report", false),
        Err(HudRouteError::BugReportUnavailable)
    );
    assert_eq!(
        controller.dispatch_route("woc.hud.pause.open.bug_report", true),
        Ok(HudRouteEffect::Host(HudHostEffect::OpenOptions(
            woc_client::OptionsPanelId::BugReport,
        )))
    );
    assert_eq!(
        controller.dispatch_route("woc.hud.pause.logout", true),
        Ok(HudRouteEffect::Host(HudHostEffect::RequestLogout))
    );
    assert_eq!(
        controller.dispatch_route("woc.hud.pause.return_to_game", true),
        Ok(HudRouteEffect::Host(HudHostEffect::SetPauseVisible {
            visible: false,
        }))
    );
    assert!(!controller.pause_visible());
}

#[test]
fn hud_route_parser_rejects_dynamic_and_out_of_range_routes() {
    for route in [
        "woc.hud.action.activate.23",
        "woc.hud.touch.activate.5",
        "woc.hud.touch.consume.6",
        "woc.hud.touch.activate.realm-row",
        "woc.hud.lockpick.engage.0",
        "woc.hud.lockpick.engage.4",
        "woc.hud.lockpick.action.q",
        "woc.shell.mode.play",
    ] {
        assert_eq!(
            parse_hud_route(route),
            Err(HudRouteError::UnknownRoute(route.to_owned()))
        );
    }
}
