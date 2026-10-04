use super::{
    RenderSurfaceHandleAllocator, RenderSurfaceHandleError, SurfaceFrameTerminal,
    SurfaceFrameTerminalHistory,
};
use crate::{DeviceGeneration, DeviceId};

#[test]
fn session_and_frame_leases_are_independently_terminalized() {
    let handles = RenderSurfaceHandleAllocator::new(DeviceId::new(1), DeviceGeneration::initial());
    let session = handles.allocate_session().unwrap();
    let frame = handles.allocate_frame().unwrap();

    handles.release_session(session).unwrap();
    assert!(matches!(
        handles.validate_session(session),
        Err(RenderSurfaceHandleError::StaleHandle { .. })
    ));
    handles.validate_frame(frame).unwrap();

    handles.release_frame(frame).unwrap();
    assert!(matches!(
        handles.validate_frame(frame),
        Err(RenderSurfaceHandleError::StaleHandle { .. })
    ));
}

#[test]
fn surface_handle_sequences_are_monotonic_and_allocator_local() {
    let handles = RenderSurfaceHandleAllocator::new(DeviceId::new(1), DeviceGeneration::initial());
    let first_session = handles.allocate_session().unwrap();
    let first_frame = handles.allocate_frame().unwrap();

    handles.release_session(first_session).unwrap();
    handles.release_frame(first_frame).unwrap();

    let second_session = handles.allocate_session().unwrap();
    let second_frame = handles.allocate_frame().unwrap();
    assert_eq!((first_session.0.value, second_session.0.value), (1, 2));
    assert_eq!((first_frame.0.value, second_frame.0.value), (1, 2));
    assert_ne!(second_session.diagnostic_id(), second_frame.diagnostic_id());
    handles.validate_session(second_session).unwrap();
    handles.validate_frame(second_frame).unwrap();

    let foreign = RenderSurfaceHandleAllocator::new(DeviceId::new(1), DeviceGeneration::initial());
    assert!(matches!(
        foreign.validate_session(second_session),
        Err(RenderSurfaceHandleError::ForeignAllocator { .. })
    ));
    assert!(matches!(
        foreign.validate_frame(second_frame),
        Err(RenderSurfaceHandleError::ForeignAllocator { .. })
    ));
}

#[test]
fn surface_handle_overflow_fails_without_publishing_a_handle() {
    let handles = RenderSurfaceHandleAllocator::new(DeviceId::new(1), DeviceGeneration::initial());
    {
        let mut state = handles
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.next_session = u64::MAX;
        state.next_frame = u64::MAX;
    }

    assert!(matches!(
        handles.allocate_session(),
        Err(RenderSurfaceHandleError::StaleHandle { .. })
    ));
    assert!(matches!(
        handles.allocate_frame(),
        Err(RenderSurfaceHandleError::StaleHandle { .. })
    ));

    let state = handles
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    assert_eq!(state.next_session, u64::MAX);
    assert_eq!(state.next_frame, u64::MAX);
    assert!(!state.active_sessions.contains(&u64::MAX));
    assert!(!state.active_frames.contains(&u64::MAX));
}

#[test]
fn terminal_history_is_bounded_but_evicted_frames_remain_allocator_stale() {
    let handles = RenderSurfaceHandleAllocator::new(DeviceId::new(1), DeviceGeneration::initial());
    let first = handles.allocate_frame().unwrap();
    let second = handles.allocate_frame().unwrap();
    handles.release_frame(first).unwrap();
    handles.release_frame(second).unwrap();

    let mut history = SurfaceFrameTerminalHistory::new(1);
    history.record(first, SurfaceFrameTerminal::Discarded);
    history.record(second, SurfaceFrameTerminal::Presented);

    assert_eq!(history.terminal(first), None);
    assert_eq!(
        history.terminal(second),
        Some(SurfaceFrameTerminal::Presented)
    );
    assert!(handles.validate_frame(first).is_err());
}
