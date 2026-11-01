//! P0-5 characterization. Production scripts and U-03 policy stay unchanged.

mod support;

use serde_json::Value;
use support::{Play, CLOCK, DRONE, FIRST_ORB, PLAYER, SCORE};
use zircon_runtime::core::math::Vec3;

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
