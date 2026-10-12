use std::time::{Duration, Instant};

use crate::core::gateway::EditorRuntimeFrameDemand;
use crate::core::play::PlayInstanceId;
use crate::ui::retained_host::ui_perf::UiPerfScenario;

#[test]
fn runtime_frame_wake_replaces_stale_requests_and_bounds_extreme_delays() {
    let host = super::UiHostWindow::new().expect("host window");
    let now = Instant::now();

    host.apply_runtime_frame_demand(EditorRuntimeFrameDemand::Continuous, now);
    assert!(host.take_external_redraw().requires_frame_update());
    assert_eq!(host.runtime_frame_wake_deadline(), None);

    host.apply_runtime_frame_demand(
        EditorRuntimeFrameDemand::SleepUntil(Duration::from_millis(25)),
        now,
    );
    assert_eq!(
        host.runtime_frame_wake_deadline(),
        Some(now + Duration::from_millis(25))
    );
    host.apply_runtime_frame_demand(EditorRuntimeFrameDemand::OnDemand, now);
    assert_eq!(host.runtime_frame_wake_deadline(), None);
    assert!(!host.take_due_runtime_frame_wake(now + Duration::from_millis(25)));

    host.apply_runtime_frame_demand(
        EditorRuntimeFrameDemand::SleepUntil(Duration::from_millis(25)),
        now,
    );
    assert!(!host.take_due_runtime_frame_wake(now));
    assert!(host.take_due_runtime_frame_wake(now + Duration::from_millis(25)));
    assert!(host.take_external_redraw().requires_frame_update());

    host.apply_runtime_frame_demand(EditorRuntimeFrameDemand::SleepUntil(Duration::MAX), now);
    assert_eq!(
        host.runtime_frame_wake_deadline(),
        Some(now + Duration::from_secs(60)),
        "an extreme transport delay must remain a bounded native wake"
    );
}

#[test]
fn runtime_frame_failure_after_consumed_wake_retries_until_success_with_capped_backoff() {
    let host = super::UiHostWindow::new().expect("host window");
    let now = Instant::now();
    let owner = (PlayInstanceId::for_test(7), 11);
    host.set_runtime_frame_owner(Some(owner));
    host.apply_runtime_frame_demand(
        EditorRuntimeFrameDemand::SleepUntil(Duration::from_millis(5)),
        now,
    );

    let mut deadline = now + Duration::from_millis(5);
    for retry_delay_ms in [50, 100, 200, 400, 800, 1_000, 1_000, 1_000, 1_000] {
        assert!(host.take_due_runtime_frame_wake(deadline));
        assert!(
            host.take_external_redraw().requires_frame_update(),
            "the host should queue and consume the due frame before the failed scene tick"
        );
        assert_eq!(
            host.complete_runtime_frame_tick(
                Err("scene tick failed"),
                Some(owner),
                Some(owner),
                deadline,
            ),
            Err("scene tick failed"),
            "the retry mechanism must preserve the original frame error"
        );

        deadline = deadline + Duration::from_millis(retry_delay_ms);
        assert_eq!(host.runtime_frame_wake_deadline(), Some(deadline));
        assert!(
            !host.take_due_runtime_frame_wake(deadline - Duration::from_nanos(1)),
            "a failed tick must not immediately spin"
        );
    }

    assert!(host.take_due_runtime_frame_wake(deadline));
    assert!(host.take_external_redraw().requires_frame_update());
    assert_eq!(
        host.complete_runtime_frame_tick::<&str>(
            Ok(EditorRuntimeFrameDemand::OnDemand),
            Some(owner),
            Some(owner),
            deadline
        ),
        Ok(())
    );
    assert_eq!(host.runtime_frame_wake_deadline(), None);
    assert!(!host.take_due_runtime_frame_wake(deadline + Duration::from_secs(2)));
}

#[test]
fn runtime_frame_owner_generation_change_cancels_stale_retry_and_resets_backoff() {
    let host = super::UiHostWindow::new().expect("host window");
    let now = Instant::now();
    let instance = PlayInstanceId::for_test(13);
    host.set_runtime_frame_owner(Some((instance, 21)));
    host.apply_runtime_frame_demand(
        EditorRuntimeFrameDemand::SleepUntil(Duration::from_millis(1)),
        now,
    );
    let first_deadline = now + Duration::from_millis(1);
    assert!(host.take_due_runtime_frame_wake(first_deadline));
    assert!(host.take_external_redraw().requires_frame_update());
    assert_eq!(
        host.complete_runtime_frame_tick(
            Err("scene tick failed"),
            Some((instance, 21)),
            Some((instance, 21)),
            first_deadline
        ),
        Err("scene tick failed")
    );
    let stale_retry_deadline = first_deadline + Duration::from_millis(50);
    assert_eq!(
        host.runtime_frame_wake_deadline(),
        Some(stale_retry_deadline)
    );

    assert!(host.take_due_runtime_frame_wake(stale_retry_deadline));
    assert!(host.take_external_redraw().requires_frame_update());
    assert_eq!(
        host.complete_runtime_frame_tick(
            Err("old gateway generation failed"),
            Some((instance, 21)),
            Some((instance, 22)),
            stale_retry_deadline
        ),
        Err("old gateway generation failed")
    );
    assert_eq!(host.runtime_frame_wake_deadline(), None);
    assert_eq!(
        host.state.borrow().runtime_frame_failure_retry_attempts,
        0,
        "a gateway generation changed during the failed pump should reset retry backoff"
    );
    assert!(!host.take_due_runtime_frame_wake(stale_retry_deadline + Duration::from_secs(2)));

    let replacement_tick = stale_retry_deadline + Duration::from_secs(2);
    host.apply_runtime_frame_demand(
        EditorRuntimeFrameDemand::SleepUntil(Duration::from_millis(1)),
        replacement_tick,
    );
    let replacement_deadline = replacement_tick + Duration::from_millis(1);
    assert!(host.take_due_runtime_frame_wake(replacement_deadline));
    assert!(host.take_external_redraw().requires_frame_update());
    assert_eq!(
        host.complete_runtime_frame_tick(
            Err("replacement scene tick failed"),
            Some((instance, 22)),
            Some((instance, 22)),
            replacement_deadline
        ),
        Err("replacement scene tick failed")
    );
    assert_eq!(
        host.runtime_frame_wake_deadline(),
        Some(replacement_deadline + Duration::from_millis(50)),
        "a replacement gateway generation should start with the base retry delay"
    );

    host.set_runtime_frame_owner(None);
    assert_eq!(host.runtime_frame_wake_deadline(), None);
    assert_eq!(host.state.borrow().runtime_frame_failure_retry_attempts, 0);
}

#[test]
fn runtime_frame_owner_change_discards_successful_stale_frame_demand() {
    let instance = PlayInstanceId::for_test(17);
    let previous_owner = Some((instance, 31));
    for next_owner in [Some((instance, 32)), None] {
        for demand in [
            EditorRuntimeFrameDemand::SleepUntil(Duration::from_millis(200)),
            EditorRuntimeFrameDemand::Continuous,
        ] {
            let host = super::UiHostWindow::new().expect("host window");
            let now = Instant::now();
            host.set_runtime_frame_owner(previous_owner);
            host.apply_runtime_frame_demand(
                EditorRuntimeFrameDemand::SleepUntil(Duration::from_millis(1)),
                now,
            );
            let deadline = now + Duration::from_millis(1);
            assert!(host.take_due_runtime_frame_wake(deadline));
            assert!(host.take_external_redraw().requires_frame_update());

            assert_eq!(
                host.complete_runtime_frame_tick::<&str>(
                    Ok(demand),
                    previous_owner,
                    next_owner,
                    deadline,
                ),
                Ok(()),
            );
            assert_eq!(host.state.borrow().runtime_frame_owner, next_owner);
            assert_eq!(host.runtime_frame_wake_deadline(), None);
            assert!(!host.take_external_redraw().requires_frame_update());
            assert!(!host.take_due_runtime_frame_wake(deadline + Duration::from_secs(1)));
        }
    }
}

#[test]
fn runtime_frame_current_owner_preserves_successful_frame_demand() {
    let host = super::UiHostWindow::new().expect("host window");
    let now = Instant::now();
    let owner = Some((PlayInstanceId::for_test(19), 41));
    host.set_runtime_frame_owner(owner);
    assert_eq!(
        host.complete_runtime_frame_tick::<&str>(
            Ok(EditorRuntimeFrameDemand::SleepUntil(Duration::from_millis(
                200
            ))),
            owner,
            owner,
            now,
        ),
        Ok(()),
    );
    assert_eq!(
        host.runtime_frame_wake_deadline(),
        Some(now + Duration::from_millis(200)),
    );
    assert_eq!(
        host.complete_runtime_frame_tick::<&str>(
            Ok(EditorRuntimeFrameDemand::Continuous),
            owner,
            owner,
            now,
        ),
        Ok(()),
    );
    assert_eq!(host.runtime_frame_wake_deadline(), None);
    assert!(host.take_external_redraw().requires_frame_update());
}

#[test]
fn maintenance_wake_queues_a_frame_update_without_visual_damage() {
    let host = super::UiHostWindow::new().expect("host window");
    let now = Instant::now();
    let deadline = now + Duration::from_millis(25);

    host.schedule_maintenance_frame_update(deadline);
    assert_eq!(host.maintenance_frame_wake_deadline(), Some(deadline));
    assert!(!host.take_due_maintenance_frame_wake(now));
    assert!(host.take_due_maintenance_frame_wake(deadline));

    let redraw = host.take_external_redraw();
    assert!(redraw.requires_frame_update());
    assert!(!redraw.requires_present());
    assert_eq!(host.maintenance_frame_wake_deadline(), None);
}

#[test]
fn lifecycle_wake_remains_independent_from_asset_maintenance() {
    let host = super::UiHostWindow::new().expect("host window");
    let now = Instant::now();
    let lifecycle_deadline = now + Duration::from_secs(5);
    let asset_deadline = now + Duration::from_millis(25);

    host.set_lifecycle_frame_update(Some(lifecycle_deadline));
    host.schedule_maintenance_frame_update(asset_deadline);
    host.clear_maintenance_frame_update();

    assert_eq!(
        host.lifecycle_frame_wake_deadline(),
        Some(lifecycle_deadline)
    );
    assert!(host.take_due_lifecycle_frame_wake(lifecycle_deadline));
    assert_eq!(
        host.take_external_redraw().scenario(),
        UiPerfScenario::SessionHeartbeat
    );
}

#[test]
fn input_timer_wake_survives_asset_maintenance_clear() {
    let host = super::UiHostWindow::new().expect("host window");
    let now = Instant::now();
    let input_deadline = now + Duration::from_millis(500);

    host.set_input_timer_frame_update(Some(input_deadline));
    host.schedule_maintenance_frame_update(now + Duration::from_millis(25));
    host.clear_maintenance_frame_update();

    assert_eq!(host.input_timer_frame_wake_deadline(), Some(input_deadline));
    assert!(host.take_due_input_timer_frame_wake(input_deadline));
    let redraw = host.take_external_redraw();
    assert!(redraw.requires_frame_update());
    assert!(!redraw.requires_present());
    assert_eq!(redraw.scenario(), UiPerfScenario::IdleHover);
}
