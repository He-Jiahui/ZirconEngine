//! Functional source fixture for the complete API-event → dynamic-session → retained-UI route.
//!
//! This fixture intentionally uses the real `UiInputEvent`, clause validator, event envelope,
//! and generation route. Its sink models the existing `RuntimeUiSurfaceSet::dispatch_input`
//! seam and the core input queue, while retaining a versioned host-output receipt so tests can
//! prove that stale callbacks do not mutate any downstream state.

use std::collections::VecDeque;

use zircon_runtime_interface::ui::dispatch::{
    UiImeInputEvent, UiImeInputEventKind, UiImePreeditClause, UiImePreeditClauseKind, UiInputEvent,
    UiInputEventMetadata, UiTextByteRange,
};
use zircon_runtime_interface::{
    ZrByteSlice, ZrRuntimeEventV1, ZrRuntimeImeCompositionContextV2,
    ZrRuntimeImeCompositionNegotiation, ZrRuntimeImeCompositionOperationV2,
    ZrRuntimeImeCompositionV2, ZrRuntimeImeCompositionWireVersion,
    ZrRuntimeImePreeditClauseAvailabilityV2, ZrRuntimeViewportHandle,
    ZIRCON_RUNTIME_ABI_VERSION_V1, ZR_RUNTIME_IME_COMPOSITION_V2_CAPABILITY,
};

use super::super::ime_composition_route::{
    RuntimeImeCompositionDispatch, RuntimeImeCompositionRoute, RuntimeImeCompositionRouteError,
};

#[derive(Clone, Debug, PartialEq, Eq)]
struct VersionedImeHostOutput {
    context: ZrRuntimeImeCompositionContextV2,
    kind: VersionedImeHostOutputKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum VersionedImeHostOutputKind {
    CompositionRange(Option<UiTextByteRange>),
    Committed,
    Cancelled,
}

#[derive(Default)]
struct RuntimeUiCompositionSink {
    committed_text: String,
    composition: Option<UiImeInputEvent>,
    input_queue: VecDeque<crate::core::framework::input::InputEvent>,
    host_outputs: Vec<VersionedImeHostOutput>,
    ui_events: Vec<UiInputEvent>,
    local_node_id: u64,
}

impl RuntimeUiCompositionSink {
    fn dispatch(
        &mut self,
        dispatch: RuntimeImeCompositionDispatch,
    ) -> Result<(), RuntimeImeCompositionRouteError> {
        let UiInputEvent::Ime(ime) = dispatch.ui_event else {
            return Err(RuntimeImeCompositionRouteError::WrongEventState);
        };
        ime.validate().map_err(|_| {
            RuntimeImeCompositionRouteError::Malformed(
                zircon_runtime_interface::ZrRuntimeImeCompositionV2Error::InvalidOperationPayload,
            )
        })?;
        let output_kind = match ime.kind {
            UiImeInputEventKind::Preedit => {
                let range = ime.cursor_range;
                self.composition = Some(ime.clone());
                VersionedImeHostOutputKind::CompositionRange(range)
            }
            UiImeInputEventKind::Commit => {
                self.committed_text.push_str(&ime.text);
                self.composition = None;
                VersionedImeHostOutputKind::Committed
            }
            UiImeInputEventKind::Cancel => {
                self.composition = None;
                VersionedImeHostOutputKind::Cancelled
            }
            UiImeInputEventKind::DeleteSurrounding => {
                return Err(RuntimeImeCompositionRouteError::WrongEventState);
            }
        };
        self.ui_events.push(UiInputEvent::Ime(ime));
        self.host_outputs.push(VersionedImeHostOutput {
            context: dispatch.context,
            kind: output_kind,
        });
        self.input_queue.push_back(dispatch.core_event);
        Ok(())
    }

    fn snapshot(&self) -> (String, Option<UiImeInputEvent>, usize, usize, usize) {
        (
            self.committed_text.clone(),
            self.composition.clone(),
            self.input_queue.len(),
            self.host_outputs.len(),
            self.ui_events.len(),
        )
    }
}

struct RuntimeImeSessionFixture {
    route: RuntimeImeCompositionRoute,
    sink: RuntimeUiCompositionSink,
}

impl RuntimeImeSessionFixture {
    fn new(window_generation: u64, local_node_id: u64) -> Self {
        let negotiation = ZrRuntimeImeCompositionNegotiation::from_peer(
            8,
            [ZR_RUNTIME_IME_COMPOSITION_V2_CAPABILITY],
        );
        assert_eq!(
            negotiation.wire_version(),
            ZrRuntimeImeCompositionWireVersion::CompositionV2
        );
        Self {
            route: RuntimeImeCompositionRoute::new(negotiation, window_generation),
            sink: RuntimeUiCompositionSink {
                local_node_id,
                ..RuntimeUiCompositionSink::default()
            },
        }
    }

    fn handle_api_event(
        &mut self,
        event: ZrRuntimeEventV1,
    ) -> Result<(), RuntimeImeCompositionRouteError> {
        let staged = self
            .route
            .consume_event(event, UiInputEventMetadata::default())?;
        self.sink.dispatch(staged)
    }
}

fn event_for(payload: &[u8]) -> ZrRuntimeEventV1 {
    ZrRuntimeEventV1::ime_composition_v2(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZrRuntimeViewportHandle::new(1),
        ZrByteSlice {
            data: payload.as_ptr(),
            len: payload.len(),
        },
    )
}

fn preedit_payload(window: u64, focus: u64, composition: u64) -> Vec<u8> {
    ZrRuntimeImeCompositionV2 {
        operation: ZrRuntimeImeCompositionOperationV2::Preedit,
        context: ZrRuntimeImeCompositionContextV2 {
            window_generation: window,
            focus_generation: focus,
            composition_generation: composition,
        },
        text: "n你🙂".to_owned(),
        cursor_range: Some(UiTextByteRange::new(4, 8)),
        clauses: vec![
            UiImePreeditClause::new(UiTextByteRange::new(0, 1), UiImePreeditClauseKind::Input),
            UiImePreeditClause::new(
                UiTextByteRange::new(1, 4),
                UiImePreeditClauseKind::Converted,
            ),
            UiImePreeditClause::new(
                UiTextByteRange::new(4, 8),
                UiImePreeditClauseKind::TargetConverted,
            ),
        ],
        clause_availability: ZrRuntimeImePreeditClauseAvailabilityV2::Available,
    }
    .encode()
    .expect("valid preedit payload")
}

fn commit_payload(window: u64, focus: u64, composition: u64) -> Vec<u8> {
    ZrRuntimeImeCompositionV2 {
        operation: ZrRuntimeImeCompositionOperationV2::Commit,
        context: ZrRuntimeImeCompositionContextV2 {
            window_generation: window,
            focus_generation: focus,
            composition_generation: composition,
        },
        text: "n你🙂".to_owned(),
        cursor_range: None,
        clauses: Vec::new(),
        clause_availability: ZrRuntimeImePreeditClauseAvailabilityV2::Unavailable,
    }
    .encode()
    .expect("valid commit payload")
}

fn cancel_payload(window: u64, focus: u64, composition: u64) -> Vec<u8> {
    ZrRuntimeImeCompositionV2 {
        operation: ZrRuntimeImeCompositionOperationV2::Cancel,
        context: ZrRuntimeImeCompositionContextV2 {
            window_generation: window,
            focus_generation: focus,
            composition_generation: composition,
        },
        text: String::new(),
        cursor_range: None,
        clauses: Vec::new(),
        clause_availability: ZrRuntimeImePreeditClauseAvailabilityV2::Unavailable,
    }
    .encode()
    .expect("valid cancel payload")
}

#[test]
fn api_event_route_applies_clauses_cursor_commit_and_versioned_host_context() {
    let mut session = RuntimeImeSessionFixture::new(701, 9);
    let preedit = preedit_payload(701, 2, 3);
    session
        .handle_api_event(event_for(&preedit))
        .expect("preedit should reach retained UI");
    let composition = session
        .sink
        .composition
        .as_ref()
        .expect("composition retained");
    assert_eq!(composition.text, "n你🙂");
    assert_eq!(composition.cursor_range, Some(UiTextByteRange::new(4, 8)));
    assert_eq!(composition.preedit_clauses.len(), 3);
    assert_eq!(session.sink.host_outputs[0].context.window_generation, 701);

    let commit = commit_payload(701, 2, 3);
    session
        .handle_api_event(event_for(&commit))
        .expect("commit should clear composition and update document");
    assert_eq!(session.sink.committed_text, "n你🙂");
    assert!(session.sink.composition.is_none());
    assert_eq!(
        session.sink.host_outputs[1].kind,
        VersionedImeHostOutputKind::Committed
    );
}

#[test]
fn malformed_and_stale_callbacks_leave_document_composition_and_host_queue_unchanged() {
    let mut session = RuntimeImeSessionFixture::new(702, 17);
    let preedit = preedit_payload(702, 4, 8);
    session
        .handle_api_event(event_for(&preedit))
        .expect("initial preedit should be accepted");
    let before = session.sink.snapshot();

    assert!(matches!(
        session.handle_api_event(event_for(&preedit[..preedit.len() - 1])),
        Err(RuntimeImeCompositionRouteError::Malformed(_))
    ));
    assert_eq!(session.sink.snapshot(), before);

    let stale = preedit_payload(702, 3, 8);
    assert_eq!(
        session.handle_api_event(event_for(&stale)),
        Err(RuntimeImeCompositionRouteError::StaleGeneration)
    );
    assert_eq!(session.sink.snapshot(), before);
}

#[test]
fn two_windows_with_reused_local_node_ids_keep_callbacks_and_documents_isolated() {
    let mut first = RuntimeImeSessionFixture::new(703, 4);
    let mut second = RuntimeImeSessionFixture::new(704, 4);
    let first_preedit = preedit_payload(703, 1, 1);
    let second_preedit = preedit_payload(704, 1, 1);
    first
        .handle_api_event(event_for(&first_preedit))
        .expect("first window preedit");
    second
        .handle_api_event(event_for(&second_preedit))
        .expect("second window preedit");

    let late_first = cancel_payload(703, 1, 1);
    assert!(first.handle_api_event(event_for(&late_first)).is_ok());
    assert!(second.sink.composition.is_some());
    assert_eq!(second.sink.host_outputs[0].context.window_generation, 704);

    let cross_window = preedit_payload(703, 2, 2);
    assert_eq!(
        second.handle_api_event(event_for(&cross_window)),
        Err(RuntimeImeCompositionRouteError::StaleGeneration)
    );
    assert_eq!(second.sink.host_outputs.len(), 1);
}
