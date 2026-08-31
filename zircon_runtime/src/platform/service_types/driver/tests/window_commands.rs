use std::num::NonZeroUsize;
use std::time::Instant;

use crate::core::framework::window::{NativeWindowId, WindowCommandTerminal};
use crate::platform::test_support::platform_driver;

use super::fixtures::{command_requested_state, command_window_state};
use crate::platform::{
    HostCommandBrokerAccessError, HostCommandBrokerError, HostCommandDispatch,
    HostWindowCommandCompletion, PlatformWindowCommandError,
};

#[test]
fn window_command_admission_updates_requested_state_only_after_the_live_target_is_validated() {
    let driver = platform_driver();
    let window = driver
        .with_window_registry(|registry| {
            registry.register(
                NativeWindowId::new(73).expect("command fixture native window is nonzero"),
            )
        })
        .expect("command fixture registers native window");
    let (create, observed, effective) = command_window_state("Initial Title");
    driver
        .with_window_states(|states| {
            states
                .register(window, create, observed, effective)
                .map(|_| ())
        })
        .expect("command fixture registers window state");
    let submitted_at = Instant::now();
    let desired = command_requested_state("Submitted Title");

    assert_eq!(
        driver.submit_window_command(
            window,
            desired.clone(),
            submitted_at + std::time::Duration::from_secs(1),
            submitted_at,
        ),
        Err(PlatformWindowCommandError::Broker(
            HostCommandBrokerAccessError::Uninstalled
        ))
    );
    assert_eq!(
        driver
            .with_window_states(|states| {
                Ok(states
                    .snapshot(window)?
                    .requested()
                    .state()
                    .title()
                    .to_owned())
            })
            .expect("failed admission leaves requested state unchanged"),
        "Initial Title"
    );

    driver
        .install_host_command_broker(
            NonZeroUsize::new(1).expect("command fixture broker limit is nonzero"),
        )
        .expect("host installs command broker");
    let accepted = driver
        .submit_window_command(
            window,
            desired,
            submitted_at + std::time::Duration::from_secs(1),
            submitted_at,
        )
        .expect("live window state and installed host admit the command");

    assert_eq!(accepted.header().target(), window);
    assert_eq!(
        driver
            .with_window_states(|states| {
                Ok(states
                    .snapshot(window)?
                    .requested()
                    .state()
                    .title()
                    .to_owned())
            })
            .expect("accepted command publishes requested state"),
        "Submitted Title"
    );
    assert_eq!(
        driver
            .with_host_command_broker(|broker| Ok(broker.pending_len()))
            .expect("accepted command enters the driver-owned broker"),
        1
    );
    assert_eq!(
        driver.submit_window_command(
            window,
            command_requested_state("Capacity Rejected Title"),
            submitted_at + std::time::Duration::from_secs(1),
            submitted_at,
        ),
        Err(PlatformWindowCommandError::Broker(
            HostCommandBrokerAccessError::Broker(HostCommandBrokerError::OutstandingLimitReached {
                limit: 1,
            })
        ))
    );
    assert_eq!(
        driver
            .with_window_states(|states| {
                Ok(states
                    .snapshot(window)?
                    .requested()
                    .state()
                    .title()
                    .to_owned())
            })
            .expect("capacity rejection leaves requested state unchanged"),
        "Submitted Title"
    );
}

#[test]
fn an_older_command_completion_never_overwrites_a_newer_requested_generation() {
    let driver = platform_driver();
    let window = driver
        .with_window_registry(|registry| {
            registry.register(
                NativeWindowId::new(74).expect("command fixture native window is nonzero"),
            )
        })
        .expect("command fixture registers native window");
    let (create, observed, effective) = command_window_state("Initial Title");
    driver
        .with_window_states(|states| {
            states
                .register(window, create, observed.clone(), effective)
                .map(|_| ())
        })
        .expect("command fixture registers window state");
    driver
        .install_host_command_broker(
            NonZeroUsize::new(2).expect("command fixture broker limit is nonzero"),
        )
        .expect("host installs command broker");

    let submitted_at = Instant::now();
    let first = driver
        .submit_window_command(
            window,
            command_requested_state("First Title"),
            submitted_at + std::time::Duration::from_secs(1),
            submitted_at,
        )
        .expect("first desired state is admitted");
    let second = driver
        .submit_window_command(
            window,
            command_requested_state("Second Title"),
            submitted_at + std::time::Duration::from_secs(1),
            submitted_at,
        )
        .expect("newer desired state is admitted");

    let first_execution = match driver
        .dispatch_next_window_command(submitted_at)
        .expect("first command dispatches through the driver transaction")
    {
        Some(HostCommandDispatch::Execute(execution)) => execution,
        other => panic!("expected executable first command, received {other:?}"),
    };
    assert_eq!(first_execution.request_id(), first.header().request_id());
    assert_ne!(
        first_execution.requested_generation(),
        driver
            .with_window_states(|states| Ok(states.snapshot(window)?.requested().generation()))
            .expect("newer requested state remains published")
    );

    let stale_receipt = driver
        .complete_window_command(
            window,
            first_execution.request_id(),
            HostWindowCommandCompletion::applied(
                observed.clone(),
                command_window_state("First Title").2,
            ),
        )
        .expect("a stale native completion still receives one receipt");
    assert!(matches!(
        stale_receipt.terminal(),
        WindowCommandTerminal::Applied
    ));
    assert_eq!(stale_receipt.effective().state().title(), "First Title");
    assert_ne!(
        stale_receipt.effective().requested_generation(),
        driver
            .with_window_states(|states| Ok(states.snapshot(window)?.requested().generation()))
            .expect("newer requested state remains published")
    );
    assert_eq!(
        driver
            .with_window_states(|states| {
                Ok(states
                    .snapshot(window)?
                    .requested()
                    .state()
                    .title()
                    .to_owned())
            })
            .expect("the newer requested state remains authoritative"),
        "Second Title"
    );
    assert_eq!(
        driver
            .with_window_states(|states| {
                Ok(states
                    .snapshot(window)?
                    .effective()
                    .state()
                    .title()
                    .to_owned())
            })
            .expect("actual effective state records the older native completion"),
        "First Title"
    );

    let second_execution = match driver
        .dispatch_next_window_command(submitted_at)
        .expect("second command dispatches through the driver transaction")
    {
        Some(HostCommandDispatch::Execute(execution)) => execution,
        other => panic!("expected executable second command, received {other:?}"),
    };
    assert_eq!(second_execution.request_id(), second.header().request_id());
    let current_receipt = driver
        .complete_window_command(
            window,
            second_execution.request_id(),
            HostWindowCommandCompletion::applied(observed, command_window_state("Second Title").2),
        )
        .expect("the newest native completion publishes effective state");
    assert_eq!(current_receipt.effective().state().title(), "Second Title");
}

#[test]
fn expired_command_keeps_requested_state_and_releases_backpressure_with_its_receipt() {
    let driver = platform_driver();
    let window = driver
        .with_window_registry(|registry| {
            registry.register(
                NativeWindowId::new(75).expect("command fixture native window is nonzero"),
            )
        })
        .expect("command fixture registers native window");
    let (create, observed, effective) = command_window_state("Initial Title");
    driver
        .with_window_states(|states| {
            states
                .register(window, create, observed, effective)
                .map(|_| ())
        })
        .expect("command fixture registers window state");
    driver
        .install_host_command_broker(
            NonZeroUsize::new(1).expect("command fixture broker limit is nonzero"),
        )
        .expect("host installs command broker");

    let submitted_at = Instant::now();
    let expired = driver
        .submit_window_command(
            window,
            command_requested_state("Expired Title"),
            submitted_at,
            submitted_at,
        )
        .expect("expired command is terminalized at admission");
    assert_eq!(
        driver
            .with_window_states(|states| {
                Ok(states
                    .snapshot(window)?
                    .requested()
                    .state()
                    .title()
                    .to_owned())
            })
            .expect("expired command did not publish requested state"),
        "Initial Title"
    );
    assert!(matches!(
        driver
            .take_window_command_receipt(expired.header().request_id())
            .expect("host consumes the terminal receipt")
            .expect("expired command retains a receipt")
            .terminal(),
        WindowCommandTerminal::Canceled
    ));

    driver
        .submit_window_command(
            window,
            command_requested_state("Live Title"),
            submitted_at + std::time::Duration::from_secs(1),
            submitted_at,
        )
        .expect("receipt consumption releases the bounded admission budget");
}
