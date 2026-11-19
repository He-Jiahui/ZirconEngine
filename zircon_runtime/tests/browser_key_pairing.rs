#![cfg(all(feature = "native-host", not(target_arch = "wasm32")))]

// Use the same source-path seam as the runtime's browser_input_validation.
// Give that file the library's real Core types; do not compile a stand-in queue.
use zircon_runtime::core;
#[allow(dead_code)]
#[path = "../src/browser/input.rs"]
mod browser_input_validation;

use crate::core::framework::input::InputManager;
use browser_input_validation::{BrowserInput, BrowserInputFrame};
use zircon_runtime::input::{
    DefaultInputManager, InputAction, InputActionEvaluator, InputActionMap, InputBinding,
    InputButton, InputEvent, InputFrameSnapshot,
};

const ACTION: &str = "qa.hold";
const SHIFT: u32 = 16;
const CONTROL: u32 = 17;
const KEY_A: u32 = 65;
const DIGIT_1: u32 = 49;
const NUMPAD_1: u32 = 97;

#[derive(serde::Deserialize)]
struct QaKeys {
    shifted: [InputEvent; 5],
    plain: [InputEvent; 3],
    control: [InputEvent; 4],
    shared_alias: [InputEvent; 4],
}

fn qa_keys() -> QaKeys {
    serde_json::from_str(include_str!("browser_key_pairing/qa_keys.json"))
        .expect("fixed listener-format InputEvent fixture")
}

#[derive(Default)]
struct Harness {
    ingress: BrowserInput,
    frame: BrowserInputFrame,
    manager: DefaultInputManager,
    raw: Vec<InputEvent>,
}

impl Harness {
    fn run_frame(&mut self, events: impl IntoIterator<Item = InputEvent>) -> InputFrameSnapshot {
        for event in events {
            self.ingress.push(event);
        }
        self.raw.clear();
        let raw = &mut self.raw;
        // This is the entry called by BrowserEngine::frame. The observer stands
        // in for the UI consumer, not for frame submission or key mapping.
        self.ingress
            .submit_frame(&mut self.frame, &self.manager, |event| {
                raw.push(event.clone());
                Ok(())
            })
            .expect("browser input submission");
        self.manager.frame_snapshot()
    }
}

fn logical(name: &str) -> InputButton {
    InputButton::Key(name.into())
}

#[track_caller]
fn expect_frame(
    frame: &InputFrameSnapshot,
    held: &[&InputButton],
    pressed: &[&InputButton],
    released: &[&InputButton],
) {
    let sorted = |buttons: &[&InputButton]| {
        let mut keys: Vec<_> = buttons.iter().copied().cloned().collect();
        keys.sort_unstable();
        keys
    };
    assert_eq!(frame.buttons.pressed_inputs(), sorted(held), "held buttons");
    assert_eq!(
        frame.buttons.just_pressed_inputs(),
        sorted(pressed),
        "press edges"
    );
    assert_eq!(
        frame.buttons.just_released_inputs(),
        sorted(released),
        "release edges"
    );
}

fn bind(button: &InputButton) -> InputActionEvaluator {
    let mut map = InputActionMap::new();
    map.add_action(InputAction::new(ACTION));
    map.bind(InputBinding::button(ACTION, button.clone()));
    InputActionEvaluator::new(map)
}

#[track_caller]
fn expect_action(
    evaluator: &InputActionEvaluator,
    frame: &InputFrameSnapshot,
    expected: [bool; 3],
) {
    let action = evaluator.evaluate(frame);
    assert_eq!(
        [
            action.pressed(ACTION),
            action.just_activated(ACTION),
            action.just_deactivated(ACTION)
        ],
        expected,
        "action [held, activated, deactivated]"
    );
}

#[test]
fn browser_key_modifier_release_clears_the_down_alias_for_one_frame() {
    let [shift_down, key_down, shift_up, _, key_up] = qa_keys().shifted;
    let mut h = Harness::default();
    let shift = logical("Shift");
    let upper = logical("A");
    let shift_code = InputButton::KeyCode(SHIFT);
    let a_code = InputButton::KeyCode(KEY_A);
    let action = bind(&upper);

    let down = h.run_frame([shift_down.clone(), key_down.clone()]);
    assert_eq!(h.raw, [shift_down, key_down]);
    let both = [&shift, &upper, &shift_code, &a_code];
    expect_frame(&down, &both, &both, &[]);
    expect_action(&action, &down, [true, true, false]);

    let shift_released = h.run_frame([shift_up]);
    expect_frame(
        &shift_released,
        &[&upper, &a_code],
        &[],
        &[&shift, &shift_code],
    );
    expect_action(&action, &shift_released, [true, false, false]);

    let released = h.run_frame([key_up.clone()]);
    assert_eq!(h.raw, [key_up], "UI must still see the lowercase keyup");
    assert!(
        !h.manager.button_pressed(&a_code),
        "physical keyup must be applied"
    );
    expect_frame(&released, &[], &[], &[&upper, &a_code]);
    assert!(!h.manager.button_pressed(&upper));
    expect_action(&action, &released, [false, false, true]);

    let next = h.run_frame([]);
    expect_frame(&next, &[], &[], &[]);
    expect_action(&action, &next, [false, false, false]);
}

#[test]
fn browser_key_changed_repeat_keeps_down_alias_and_raw_text() {
    let [shift_down, key_down, shift_up, changed_repeat, key_up] = qa_keys().shifted;
    let mut h = Harness::default();
    let upper = logical("A");
    let a_code = InputButton::KeyCode(KEY_A);
    h.run_frame([shift_down, key_down]);
    h.run_frame([shift_up]);

    let repeated = h.run_frame([changed_repeat.clone()]);
    assert_eq!(
        h.raw,
        [changed_repeat],
        "raw UI receives text=a and repeat=true"
    );
    expect_frame(&repeated, &[&upper, &a_code], &[], &[]);

    let released = h.run_frame([key_up]);
    expect_frame(&released, &[], &[], &[&upper, &a_code]);
    expect_frame(&h.run_frame([]), &[], &[], &[]);
}

#[test]
fn browser_key_unmodified_and_control_pairs_keep_normal_edges() {
    let QaKeys {
        plain: [down, repeat, up],
        control: [ctrl_down, ctrl_a_down, ctrl_a_up, ctrl_up],
        ..
    } = qa_keys();
    let mut h = Harness::default();
    let lower = logical("a");
    let ctrl = logical("Control");
    let a_code = InputButton::KeyCode(KEY_A);
    let ctrl_code = InputButton::KeyCode(CONTROL);
    let a = [&a_code, &lower];

    expect_frame(&h.run_frame([down]), &a, &a, &[]);
    expect_frame(&h.run_frame([repeat]), &a, &[], &[]);
    expect_frame(&h.run_frame([up]), &[], &[], &a);
    expect_frame(&h.run_frame([]), &[], &[], &[]);

    let both = [&ctrl_code, &ctrl, &a_code, &lower];
    expect_frame(&h.run_frame([ctrl_down, ctrl_a_down]), &both, &both, &[]);
    expect_frame(&h.run_frame([ctrl_a_up]), &[&ctrl_code, &ctrl], &[], &a);
    expect_frame(&h.run_frame([ctrl_up]), &[], &[], &[&ctrl_code, &ctrl]);
    expect_frame(&h.run_frame([]), &[], &[], &[]);
}

#[test]
fn browser_key_shared_alias_releases_only_after_both_physical_keys() {
    let [digit_down, numpad_down, digit_up, numpad_up] = qa_keys().shared_alias;
    let mut h = Harness::default();
    let digit = InputButton::KeyCode(DIGIT_1);
    let pad = InputButton::KeyCode(NUMPAD_1);
    let alias = logical("1");
    let action = bind(&alias);

    let first = h.run_frame([digit_down]);
    expect_frame(&first, &[&digit, &alias], &[&digit, &alias], &[]);
    expect_action(&action, &first, [true, true, false]);

    let both = h.run_frame([numpad_down]);
    expect_frame(&both, &[&digit, &pad, &alias], &[&pad], &[]);
    expect_action(&action, &both, [true, false, false]);

    let one_left = h.run_frame([digit_up.clone()]);
    assert_eq!(h.raw, [digit_up]);
    assert!(!h.manager.button_pressed(&digit));
    assert!(h.manager.button_pressed(&pad));
    expect_frame(&one_left, &[&pad, &alias], &[], &[&digit]);
    expect_action(&action, &one_left, [true, false, false]);

    let last = h.run_frame([numpad_up]);
    expect_frame(&last, &[], &[], &[&pad, &alias]);
    expect_action(&action, &last, [false, false, true]);
    let next = h.run_frame([]);
    expect_frame(&next, &[], &[], &[]);
    expect_action(&action, &next, [false, false, false]);
}

#[test]
fn browser_key_focus_loss_and_overflow_reset_before_the_new_down() {
    const QUEUE_CAPACITY: usize = 1024;
    let a_code = InputButton::KeyCode(KEY_A);
    let upper = logical("A");
    let lower = logical("a");
    for overflow in [false, true] {
        let QaKeys {
            shifted: [shift, upper_down, _, _, _],
            plain: [lower_down, _, lower_up],
            ..
        } = qa_keys();
        let mut h = Harness::default();
        let shift_code = InputButton::KeyCode(SHIFT);
        let shift_alias = logical("Shift");
        let previous = [&a_code, &upper, &shift_code, &shift_alias];
        expect_frame(&h.run_frame([shift, upper_down]), &previous, &previous, &[]);

        // Make the new key the event that survives overflow. Both sources of
        // FocusLost must clear old pairings before the next event of this frame.
        let reset = if overflow {
            h.run_frame(
                std::iter::repeat_n(InputEvent::CursorEntered, QUEUE_CAPACITY)
                    .chain([lower_down.clone()]),
            )
        } else {
            h.run_frame([InputEvent::FocusLost, lower_down.clone()])
        };
        assert_eq!(
            h.ingress.omitted_events(),
            if overflow { QUEUE_CAPACITY as u64 } else { 0 }
        );
        assert_eq!(
            h.raw,
            [InputEvent::FocusLost, lower_down],
            "overflow={overflow}"
        );
        let current = [&a_code, &lower];
        expect_frame(&reset, &current, &current, &previous);
        expect_frame(&h.run_frame([]), &current, &[], &[]);
        expect_frame(&h.run_frame([lower_up]), &[], &[], &current);
        expect_frame(&h.run_frame([]), &[], &[], &[]);
    }
}

#[test]
fn browser_key_legacy_none_is_physical_and_utf8_bounds_precede_pairing() {
    // These are explicit ingress boundary values, not DOM samples.
    fn keyboard(code: u32, alias: Option<&str>, text: Option<&str>, pressed: bool) -> InputEvent {
        InputEvent::KeyboardInput {
            key_code: code,
            logical_key: alias.map(str::to_owned),
            text: text.map(str::to_owned),
            pressed,
            repeat: false,
        }
    }
    let mut h = Harness::default();
    let bounded_alias = "x".repeat(127);
    let bounded_text = "y".repeat(126);
    let wide_alias = format!("{bounded_alias}界");
    let wide_text = format!("{bounded_text}界!");
    let named_code = InputButton::KeyCode(88);
    let legacy_code = InputButton::KeyCode(89);
    let alias = logical(&bounded_alias);
    let named = [&named_code, &alias];

    let capped = h.run_frame([keyboard(88, Some(&wide_alias), Some(&wide_text), true)]);
    assert_eq!(
        h.raw,
        [keyboard(
            88,
            Some(&bounded_alias),
            Some(&bounded_text),
            true
        )]
    );
    expect_frame(&capped, &named, &named, &[]);

    let legacy_down = keyboard(89, None, Some("y"), true);
    let mixed = h.run_frame([legacy_down.clone()]);
    assert_eq!(h.raw, [legacy_down]);
    expect_frame(
        &mixed,
        &[&named_code, &legacy_code, &alias],
        &[&legacy_code],
        &[],
    );
    let legacy_up = keyboard(89, None, Some("y"), false);
    let physical_released = h.run_frame([legacy_up.clone()]);
    assert_eq!(h.raw, [legacy_up]);
    expect_frame(&physical_released, &named, &[], &[&legacy_code]);

    let named_up = h.run_frame([keyboard(88, Some(&wide_alias), None, false)]);
    expect_frame(&named_up, &[], &[], &named);
    expect_frame(&h.run_frame([]), &[], &[], &[]);
}
