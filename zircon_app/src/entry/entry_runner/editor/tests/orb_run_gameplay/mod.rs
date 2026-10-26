//! P0-5 characterization. Production scripts and U-03 policy stay unchanged.

mod support;

use serde_json::{json, Value};
use support::{Play, CLOCK, FIRST_ORB, PLAYER, SCORE};
use zircon_runtime::core::math::Vec3;

#[test]
fn start_w_release_and_hp_write_characterize_menu_bounce() {
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
    assert_eq!(
        play.state(),
        "start_menu",
        "characterization: hp invalidation reruns onStart"
    );
    assert_eq!(
        play.component(PLAYER, "gameplay.menu_state")["title"],
        "Orb Run"
    );
    println!(
        "P0_5_MENU_BOUNCE root={} hp={} z={}",
        play.root.display(),
        play.hp(CLOCK),
        moved.z
    );
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
}

#[test]
fn pickup_and_three_seeded_retry_clicks_reset_score_hp_and_all_orbs() {
    let play = Play::example("pickup-retry");
    play.start();
    for retry in 1..=3 {
        // Deterministic proximity setup; pickup/score changes execute in the real VM.
        play.set_position(PLAYER, play.position(FIRST_ORB));
        play.tick();
        assert_eq!(play.component(FIRST_ORB, "orb_run.collected"), "1");
        assert_eq!(play.position(FIRST_ORB), Vec3::new(0.0, -50.0, 0.0));
        assert_eq!(play.hp(SCORE), 2.0);
        play.tick();
        assert_eq!(play.state(), "start_menu");
        // The restart defect prevents a natural completed run. Seed only the
        // game-over menu to test the native Retry writer and immediate reset.
        play.set_component(
            PLAYER,
            "gameplay.menu_state",
            json!({"state":"game_over","button":"Play again"}),
        );
        play.set_component(PLAYER, "orb_run.run_state", json!("ended"));
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
        assert_eq!(
            play.state(),
            "start_menu",
            "score reset also invalidates lifecycle"
        );
        // Characterize retry above, then use a real Start to enter the next round.
        play.click();
        play.tick();
        assert_eq!(play.state(), "reset");
        play.tick();
        assert_eq!(play.state(), "playing");
        println!(
            "P0_5_SEEDED_RETRY round={retry} score_hp={} clock_hp={}",
            play.hp(SCORE),
            play.hp(CLOCK)
        );
    }
}

#[test]
fn seeded_timeout_ends_without_destroying_the_clock() {
    let play = Play::example("timeout");
    play.tick();
    play.set_hp(CLOCK, 1.0);
    play.tick(); // Consume the binding invalidation before seeding the playing state.
    play.set_component(PLAYER, "orb_run.run_state", json!("playing"));
    play.set_component(PLAYER, "gameplay.menu_state", Value::Null);
    play.tick();
    assert_eq!(play.state(), "ended");
    assert_eq!(
        play.component(PLAYER, "gameplay.menu_state")["title"],
        "Time's up"
    );
    assert_eq!(play.hp(CLOCK), 1.0);
    assert!(play.read(|world| world.local_transform(CLOCK).is_some()));
}

#[test]
fn u03_native_host_start_error_is_swallowed_and_later_bindings_run() {
    let play = Play::failing_binding("start");
    for attempt in 1..=3 {
        play.tick();
        for entity in 30..=32 {
            assert_eq!(play.position(entity), Vec3::new(1.0, attempt as f32, 0.0));
        }
        assert_eq!(play.component(31, "b3_probe.marker"), Value::Null);
        println!("P0_5_U03_HOST_START attempt={attempt} tick=Ok later_binding_ran=true");
    }
}

#[test]
fn u03_native_host_update_error_is_swallowed_and_later_bindings_run() {
    let play = Play::failing_binding("update");
    for attempt in 1..=3 {
        play.tick();
        for entity in 30..=32 {
            assert_eq!(play.position(entity), Vec3::new(1.0, attempt as f32, 0.0));
        }
        assert_eq!(play.component(31, "b3_probe.marker"), Value::Null);
        println!("P0_5_U03_HOST_UPDATE attempt={attempt} tick=Ok later_binding_ran=true");
    }
}

#[test]
fn u03_native_host_fixed_error_hits_generation_guard_and_keeps_world_writes() {
    let play = Play::failing_binding("fixed");
    for attempt in 1..=3 {
        std::thread::sleep(std::time::Duration::from_millis(20));
        let error = play
            .tick_result()
            .expect_err("fixed-stage world writes trip the generation guard");
        assert!(
            error.contains("World generation changed during FixedUpdate"),
            "{error}"
        );
        for entity in 30..=32 {
            assert_eq!(play.position(entity), Vec3::new(0.0, 0.0, attempt as f32));
        }
        assert_eq!(play.fixed_elapsed(), std::time::Duration::ZERO);
        println!("P0_5_U03_HOST_FIXED attempt={attempt} error={error}");
    }
}

#[test]
fn u03_missing_start_binding_aborts_later_bindings_and_retries_each_frame() {
    let play = Play::missing_binding(false);
    for attempt in 1..=3 {
        let error = play
            .tick_result()
            .expect_err("binding-resolution errors fail the native tick");
        assert!(
            error.contains("onStart")
                && error.contains("b3_missing")
                && error.contains("not loaded"),
            "{error}"
        );
        assert_eq!(play.position(30), Vec3::new(1.0, attempt as f32, 0.0));
        assert_eq!(play.position(31), Vec3::ZERO);
        assert_eq!(play.position(32), Vec3::ZERO, "later binding never starts");
        println!("P0_5_U03_BINDING_START attempt={attempt} error={error}");
    }
}

#[test]
fn u03_missing_fixed_binding_aborts_phase_but_keeps_prior_world_writes() {
    let play = Play::missing_binding(true);
    for attempt in 1..=3 {
        std::thread::sleep(std::time::Duration::from_millis(20));
        let error = play
            .tick_result()
            .expect_err("binding-resolution errors fail the fixed step");
        assert!(
            error.contains("onFixedUpdate")
                && error.contains("b3_missing")
                && error.contains("not loaded"),
            "{error}"
        );
        assert_eq!(play.position(30), Vec3::new(0.0, 0.0, attempt as f32));
        assert_eq!(play.position(31), Vec3::ZERO);
        assert_eq!(play.position(32), Vec3::ZERO);
        // The aborted transaction settles fixed time, not the World mutations.
        assert_eq!(play.fixed_elapsed(), std::time::Duration::ZERO);
        println!("P0_5_U03_BINDING_FIXED attempt={attempt} error={error}");
    }
}
