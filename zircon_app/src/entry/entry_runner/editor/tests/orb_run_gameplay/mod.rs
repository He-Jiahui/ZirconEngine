//! Native Orb Run continuity and U-03 per-binding failure isolation.

mod support;

use serde_json::Value;
use support::{Play, CLOCK, DRONE, FIRST_ORB, PLAYER, SCORE};
use zircon_runtime::core::math::Vec3;
use zircon_runtime::scene::{ScriptBindingFault, ScriptBindingFaultKind};

#[test]
fn start_w_release_and_clock_hp_write_keep_playing() {
    let play = Play::example("start-w-hp");
    play.start();
    assert_eq!(play.hp(CLOCK), 61.0);
    assert_eq!(play.hp(SCORE), 1.0);
    let before = play.position(PLAYER);
    let generation = play.read(|world| world.dynamic_component_generation("script.bindings"));
    play.key(b'W', true);
    play.tick();
    assert!(
        play.position(PLAYER).z < before.z,
        "native W must move Player toward -Z"
    );
    assert_eq!(play.state(), "playing");
    assert!(play.hp(CLOCK) < 61.0 && play.hp(CLOCK) > 60.0);
    assert!(play.read(|world| world.dynamic_component_generation("script.bindings")) > generation);
    let moved = play.position(PLAYER);
    play.key(b'W', false);
    play.tick();
    assert_eq!(play.position(PLAYER), moved);
    assert_eq!(play.state(), "playing");
    assert_eq!(play.component(PLAYER, "gameplay.menu_state"), Value::Null);
    println!(
        "P0_5_PLAYING_AFTER_HP_WRITE root={} hp={} z={}",
        play.root.display(),
        play.hp(CLOCK),
        moved.z
    );
    play.assert_no_script_faults();
}

#[test]
fn red_playing_must_survive_the_clock_hp_write() {
    let play = Play::example("red-hp-restart");
    play.start();
    play.tick();
    assert!(
        play.hp(CLOCK) < 61.0,
        "must exercise damage_entity before the regression assertion"
    );
    play.tick();
    // Deliberately RED until Claude's separate hp/binding-lifecycle unit. Do not
    // weaken this to the observed menu state or mix the product fix into B3.
    assert_eq!(
        play.state(),
        "playing",
        "P0-5: clock hp writes must not restart Player onStart"
    );
    assert_eq!(play.component(PLAYER, "gameplay.menu_state"), Value::Null);
    play.assert_no_script_faults();
}

#[test]
fn p_key_edge_pauses_movement_and_clock_then_resumes() {
    let play = Play::example("pause");
    play.start();
    let before = play.position(PLAYER);
    play.key(b'P', true);
    play.key(b'W', true);
    play.tick();
    assert_eq!(play.state(), "paused");
    assert_eq!(play.component(PLAYER, "orb_run.p_was_down"), "1");
    for _ in 0..3 {
        play.tick();
    }
    assert_eq!(play.state(), "paused", "held P must not toggle every frame");
    assert_eq!(play.position(PLAYER), before);
    assert_eq!(play.hp(CLOCK), 61.0);
    play.key(b'P', false);
    play.tick();
    assert_eq!(play.component(PLAYER, "orb_run.p_was_down"), "0");
    play.key(b'P', true);
    play.tick();
    assert_eq!(play.state(), "playing");
    play.key(b'P', false);
    play.key(b'W', false);
    play.tick();
    assert_eq!(play.state(), "playing");
    assert_eq!(
        play.position(PLAYER),
        before,
        "W up must stop movement while playing"
    );
    assert!(play.hp(CLOCK) < 61.0, "resumed play must advance the timer");
    play.assert_no_script_faults();
}

#[test]
fn pickup_and_three_natural_game_overs_retry_and_keep_playing() {
    let play = Play::example("pickup-retry");
    play.start();
    for retry in 1..=3 {
        // Deterministic proximity setup; pickup/score changes execute in the real VM.
        play.set_position(PLAYER, play.position(FIRST_ORB));
        play.tick();
        assert_eq!(play.component(FIRST_ORB, "orb_run.collected"), "1");
        assert_eq!(play.position(FIRST_ORB), Vec3::new(0.0, -50.0, 0.0));
        assert_eq!(play.hp(SCORE), 2.0);
        let title = match retry {
            1 => {
                play.set_hp(CLOCK, 1.0);
                play.tick();
                "Time's up"
            }
            2 => {
                play.set_position(PLAYER, play.position(DRONE));
                play.tick();
                "Caught!"
            }
            3 => {
                for orb in FIRST_ORB + 1..=17 {
                    play.set_position(PLAYER, play.position(orb));
                    play.tick();
                    assert_eq!(play.component(orb, "orb_run.collected"), "1");
                }
                assert_eq!(play.hp(SCORE), 9.0);
                // Player checks the winning score before the orbs update.
                play.tick();
                "You win!"
            }
            _ => unreachable!(),
        };
        assert_eq!(play.state(), "ended");
        assert_eq!(
            play.component(PLAYER, "gameplay.menu_state")["title"],
            title
        );
        assert_eq!(
            play.component(PLAYER, "gameplay.menu_state")["state"],
            "game_over"
        );
        play.click();
        assert_eq!(
            play.component(PLAYER, "gameplay.control_state"),
            "retry_game"
        );
        play.tick();
        assert_eq!(play.state(), "reset");
        assert_eq!(play.position(PLAYER), Vec3::new(0.0, 0.5, 0.0));
        assert_eq!(play.hp(SCORE), 1.0);
        assert_eq!(play.hp(CLOCK), 61.0);
        for orb in 10..=17 {
            assert_eq!(play.component(orb, "orb_run.collected"), "0");
            let props = &play.component(orb, "script.bindings")[0]["properties"];
            assert_eq!(
                play.position(orb),
                Vec3::new(
                    props["home_x"].as_f64().unwrap() as f32,
                    props["home_y"].as_f64().unwrap() as f32,
                    props["home_z"].as_f64().unwrap() as f32,
                )
            );
        }
        play.tick();
        assert_eq!(play.state(), "playing");
        assert_eq!(play.component(PLAYER, "gameplay.menu_state"), Value::Null);
        play.tick();
        assert_eq!(play.state(), "playing");
        assert_eq!(play.component(PLAYER, "gameplay.menu_state"), Value::Null);
        println!(
            "P0_5_NATURAL_RETRY round={retry} ending={title} score_hp={} clock_hp={}",
            play.hp(SCORE),
            play.hp(CLOCK)
        );
    }
    play.assert_no_script_faults();
}

#[test]
fn natural_start_clock_seed_times_out_without_destroying_the_clock() {
    let play = Play::example("timeout");
    play.start();
    play.set_hp(CLOCK, 1.0);
    play.tick();
    assert_eq!(play.state(), "ended");
    assert_eq!(
        play.component(PLAYER, "gameplay.menu_state")["title"],
        "Time's up"
    );
    assert_eq!(play.hp(CLOCK), 1.0);
    assert!(play.read(|world| world.local_transform(CLOCK).is_some()));
    play.tick();
    assert_eq!(play.state(), "ended");
    play.assert_no_script_faults();
}

#[test]
fn sustained_w_hold_keeps_playing_and_drains_clock() {
    let play = Play::example("sustained-w");
    play.start();
    play.key(b'W', true);
    let mut previous_hp = play.hp(CLOCK);
    let mut previous_z = play.position(PLAYER).z;
    for tick in 1..=30 {
        play.tick();
        assert_eq!(play.state(), "playing", "held W tick {tick}");
        assert_eq!(play.component(PLAYER, "gameplay.menu_state"), Value::Null);
        let hp = play.hp(CLOCK);
        let z = play.position(PLAYER).z;
        assert!(hp < previous_hp, "clock must drain on tick {tick}");
        assert!(z <= previous_z && z >= -11.0, "Player z={z} on tick {tick}");
        previous_hp = hp;
        previous_z = z;
    }
    play.assert_no_script_faults();
}

fn isolated_fault(play: &Play, package: &str, export: &str) -> ScriptBindingFault {
    assert_eq!(play.script_fault_total(), 1);
    let faults = play.script_faults();
    assert_eq!(faults.len(), 1);
    let fault = faults.into_iter().next().unwrap();
    assert_eq!(fault.entity, 31);
    assert_eq!(
        fault.binding_key.as_deref(),
        Some(format!("{package}::main#0").as_str())
    );
    assert_eq!(fault.package, package);
    assert_eq!(fault.module, "main");
    assert_eq!(fault.export, export);
    assert!(fault.outer_frame_index > 0);
    assert!(
        zircon_runtime::diagnostic_log::flush_process_log(std::time::Duration::from_secs(2)),
        "retain the asynchronous binding diagnostic before the test process exits"
    );
    fault
}

fn assert_json_host_fault(play: &Play, export: &str) {
    let fault = isolated_fault(play, "b3_probe", export);
    assert_eq!(
        fault.kind,
        ScriptBindingFaultKind::HostCall {
            module: "zr.zircon.gameplay".into(),
            function: "set_component_json".into(),
        }
    );
    assert_eq!(fault.host_fault_count, 1);
    assert!(
        fault.message.contains("EOF while parsing an object"),
        "{}",
        fault.message
    );
    assert_eq!(play.component(31, "b3_probe.marker"), Value::Null);
}

#[test]
fn u03_host_call_error_in_start_kills_only_that_binding() {
    let play = Play::failing_binding("start");
    for attempt in 1..=3 {
        play.tick();
        for entity in [30, 32] {
            assert_eq!(play.position(entity), Vec3::new(1.0, attempt as f32, 0.0));
        }
        assert_eq!(play.position(31), Vec3::new(1.0, 0.0, 0.0));
        assert_json_host_fault(&play, "onStart");
    }
}

#[test]
fn u03_host_call_error_in_update_kills_only_that_binding() {
    let play = Play::failing_binding("update");
    for attempt in 1..=3 {
        play.tick();
        for entity in [30, 32] {
            assert_eq!(play.position(entity), Vec3::new(1.0, attempt as f32, 0.0));
        }
        assert_eq!(play.position(31), Vec3::new(1.0, 1.0, 0.0));
        assert_json_host_fault(&play, "onUpdate");
    }
}

#[test]
fn u03_host_call_error_in_fixed_kills_only_that_binding_guard_pending() {
    let play = Play::failing_binding("fixed");
    for attempt in 1..=3 {
        std::thread::sleep(std::time::Duration::from_millis(20));
        let error = play
            .tick_result()
            .expect_err("Unit 3 still owns the fixed-stage generation guard");
        assert!(
            error.contains("World generation changed during FixedUpdate"),
            "{error}"
        );
        for entity in [30, 32] {
            assert_eq!(play.position(entity), Vec3::new(0.0, 0.0, attempt as f32));
        }
        assert_eq!(play.position(31), Vec3::new(0.0, 0.0, 1.0));
        assert_eq!(play.fixed_elapsed(), std::time::Duration::ZERO);
        assert_json_host_fault(&play, "onFixedUpdate");
    }
}

#[test]
fn u03_missing_start_binding_is_isolated_and_not_retried() {
    let play = Play::missing_binding(false);
    for attempt in 1..=3 {
        play.tick();
        for entity in [30, 32] {
            assert_eq!(play.position(entity), Vec3::new(1.0, attempt as f32, 0.0));
        }
        assert_eq!(play.position(31), Vec3::ZERO);
        let fault = isolated_fault(&play, "b3_missing", "onStart");
        assert_eq!(fault.kind, ScriptBindingFaultKind::Resolution);
        assert_eq!(fault.host_fault_count, 0);
        assert!(
            fault.message.contains("b3_missing") && fault.message.contains("not loaded"),
            "{}",
            fault.message
        );
    }
}

#[test]
fn u03_missing_fixed_binding_is_isolated_guard_pending() {
    let play = Play::missing_binding(true);
    for attempt in 1..=3 {
        std::thread::sleep(std::time::Duration::from_millis(20));
        let error = play
            .tick_result()
            .expect_err("Unit 3 still owns the fixed-stage generation guard");
        assert!(
            error.contains("World generation changed during FixedUpdate"),
            "{error}"
        );
        for entity in [30, 32] {
            assert_eq!(play.position(entity), Vec3::new(0.0, 0.0, attempt as f32));
        }
        assert_eq!(play.position(31), Vec3::ZERO);
        assert_eq!(play.fixed_elapsed(), std::time::Duration::ZERO);
        let fault = isolated_fault(&play, "b3_missing", "onFixedUpdate");
        assert_eq!(fault.kind, ScriptBindingFaultKind::Resolution);
        assert_eq!(fault.host_fault_count, 0);
        assert!(
            fault.message.contains("b3_missing") && fault.message.contains("not loaded"),
            "{}",
            fault.message
        );
    }
}
