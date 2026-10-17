//! 复用父模块夹具，核对输入泵的相邻重绘合并、平台规范化与显式 IME 范围准入。

use super::*;

/// 两个相邻重绘保留首项，后续光标与 IME 保持顺序；再核对整批 JSON 往返。
#[test]
fn ui_window_input_pump_wraps_window_and_shared_input_events_with_redraw_coalescing() {
    let metadata = sample_window_metadata();
    let redraw = UiWindowInputPumpEvent::Window(UiWindowEvent::new(
        metadata.clone(),
        UiWindowEventKind::RequestRedraw {
            reason: UiWindowRedrawReason::Input,
        },
    ));
    let second_redraw = UiWindowInputPumpEvent::Window(UiWindowEvent::new(
        metadata.clone(),
        UiWindowEventKind::RequestRedraw {
            reason: UiWindowRedrawReason::Animation,
        },
    ));
    let cursor_left =
        UiWindowInputPumpEvent::Window(UiWindowEvent::new(metadata, UiWindowEventKind::CursorLeft));
    let ime = UiWindowInputPumpEvent::Input(UiInputEvent::Ime(UiImeInputEvent {
        metadata: sample_input_metadata(),
        kind: UiImeInputEventKind::Preedit,
        text: "draft".to_string(),
        cursor_range: Some(UiTextByteRange::new(0, 5)),
        preedit_clauses: Vec::new(),
        delete_surrounding: None,
    }));

    let mut batch = UiWindowInputPumpBatch::default();
    batch.push_coalesced(redraw.clone());
    batch.push_coalesced(second_redraw);
    batch.push_coalesced(cursor_left.clone());
    batch.push_coalesced(ime.clone());

    assert_eq!(batch.events.len(), 3);
    assert_eq!(batch.events[0], redraw);
    assert_eq!(batch.events[1], cursor_left);
    assert_eq!(batch.events[2], ime);
    assert!(matches!(batch.events[2], UiWindowInputPumpEvent::Input(_)));
    assert_eq!(round_trip(&batch), batch);
}

/// 经泵入口规范化平台事件，保留预编辑范围与来源元数据，并以触摸身份覆盖上下文指针。
#[test]
fn ui_window_input_pump_accepts_platform_input_events_through_normalization() {
    let metadata = sample_window_metadata().synthetic(true);
    let context = UiWindowInputContext::from_window_metadata(&metadata)
        .with_device_id(UiDeviceId::new(9))
        .with_pointer_id(UiPointerId::new(11));

    let mut batch = UiWindowInputPumpBatch::default();
    batch.push_platform_input(UiWindowPlatformInputEvent::ime_with_cursor_range(
        context.clone(),
        UiImeInputEventKind::Preedit,
        "draft",
        Some(UiTextByteRange::new(1, 4)),
    ));
    batch.push_platform_input(UiWindowPlatformInputEvent::touch(
        context,
        UiWindowTouchPhase::Moved,
        UiPointerId::new(77),
        UiPoint::new(14.0, 28.0),
    ));

    assert_eq!(batch.events.len(), 2);
    assert!(matches!(
        &batch.events[0],
        UiWindowInputPumpEvent::Input(UiInputEvent::Ime(ime))
            if ime.kind == UiImeInputEventKind::Preedit
                && ime.text == "draft"
                && ime.cursor_range == Some(UiTextByteRange::new(1, 4))
                && ime.preedit_clauses.is_empty()
                && ime.delete_surrounding.is_none()
                && ime.metadata.window_id == Some(metadata.window_id.clone())
                && ime.metadata.device_id == Some(UiDeviceId::new(9))
                && ime.metadata.synthetic
    ));
    assert!(matches!(
        &batch.events[1],
        UiWindowInputPumpEvent::Input(UiInputEvent::Pointer(pointer))
            if pointer.metadata.window_id == Some(metadata.window_id.clone())
                && pointer.metadata.device_id == Some(UiDeviceId::new(9))
                && pointer.metadata.pointer_id == Some(UiPointerId::new(77))
                && pointer.metadata.pointer_source == UiPointerSource::Touch
                && pointer.event.kind == UiPointerEventKind::Move
                && pointer.event.point == UiPoint::new(14.0, 28.0)
                && pointer.metadata.synthetic
    ));
    assert_eq!(round_trip(&batch), batch);
}

/// 确认经构造校验的光标和子句进入规范化结果，并保留转换态的序列化标签。
#[test]
fn ui_window_input_pump_preserves_validated_preedit_clauses() {
    let metadata = sample_window_metadata().synthetic(true);
    let context = UiWindowInputContext::from_window_metadata(&metadata);
    let clauses = vec![
        UiImePreeditClause::new(UiTextByteRange::new(0, 1), UiImePreeditClauseKind::Input),
        UiImePreeditClause::new(
            UiTextByteRange::new(1, 2),
            UiImePreeditClauseKind::Converted,
        ),
        UiImePreeditClause::new(
            UiTextByteRange::new(2, 3),
            UiImePreeditClauseKind::TargetConverted,
        ),
        UiImePreeditClause::new(
            UiTextByteRange::new(3, 4),
            UiImePreeditClauseKind::TargetNotConverted,
        ),
    ];

    let input = UiWindowPlatformInputEvent::ime_with_preedit_clauses(
        context,
        "abcd",
        Some(UiTextByteRange::new(3, 4)),
        clauses.clone(),
    )
    .expect("preedit clauses must use valid UTF-8 byte ranges")
    .normalize();

    assert!(matches!(
        &input,
        UiInputEvent::Ime(ime)
            if ime.kind == UiImeInputEventKind::Preedit
                && ime.text == "abcd"
                && ime.cursor_range == Some(UiTextByteRange::new(3, 4))
                && ime.preedit_clauses == clauses
    ));
    let encoded = serde_json::to_string(&input).expect("serialize preedit clauses");
    assert!(encoded.contains("\"target_converted\""));
    assert_eq!(round_trip(&input), input);
}

/// 覆盖非 UTF-8 边界、越界、重叠与乱序；除乱序外，此处仅断言构造失败。
#[test]
fn ui_window_input_pump_rejects_invalid_preedit_clause_ranges() {
    let context = UiWindowInputContext::from_window_metadata(&sample_window_metadata());

    let non_boundary = UiWindowPlatformInputEvent::ime_with_preedit_clauses(
        context.clone(),
        "\u{4e2d}",
        None,
        vec![UiImePreeditClause::new(
            UiTextByteRange::new(1, 3),
            UiImePreeditClauseKind::Converted,
        )],
    );
    assert!(non_boundary.is_err());

    let invalid_cursor = UiWindowPlatformInputEvent::ime_with_preedit_clauses(
        context.clone(),
        "\u{4e2d}",
        Some(UiTextByteRange::new(1, 3)),
        Vec::new(),
    );
    assert!(invalid_cursor.is_err());

    let outside_text = UiWindowPlatformInputEvent::ime_with_preedit_clauses(
        context,
        "ok",
        None,
        vec![UiImePreeditClause::new(
            UiTextByteRange::new(0, 3),
            UiImePreeditClauseKind::TargetNotConverted,
        )],
    );
    assert!(outside_text.is_err());

    let overlapping = UiWindowPlatformInputEvent::ime_with_preedit_clauses(
        UiWindowInputContext::from_window_metadata(&sample_window_metadata()),
        "ok",
        None,
        vec![
            UiImePreeditClause::new(UiTextByteRange::new(0, 2), UiImePreeditClauseKind::Input),
            UiImePreeditClause::new(
                UiTextByteRange::new(1, 2),
                UiImePreeditClauseKind::Converted,
            ),
        ],
    );
    assert!(overlapping.is_err());

    let unordered = UiWindowPlatformInputEvent::ime_with_preedit_clauses(
        UiWindowInputContext::from_window_metadata(&sample_window_metadata()),
        "abc",
        None,
        vec![
            UiImePreeditClause::new(UiTextByteRange::new(2, 3), UiImePreeditClauseKind::Input),
            UiImePreeditClause::new(
                UiTextByteRange::new(0, 1),
                UiImePreeditClauseKind::Converted,
            ),
        ],
    );
    assert!(matches!(
        unordered,
        Err(UiImePreeditClauseError::RangeSequenceOutOfOrder)
    ));
}

/// 直接构造提交事件后显式校验，确认预编辑子句不能附在非预编辑种类上。
#[test]
fn ui_ime_input_event_rejects_clauses_outside_preedit() {
    let event = UiImeInputEvent {
        metadata: sample_input_metadata(),
        kind: UiImeInputEventKind::Commit,
        text: "x".to_string(),
        cursor_range: None,
        preedit_clauses: vec![UiImePreeditClause::new(
            UiTextByteRange::new(0, 1),
            UiImePreeditClauseKind::Converted,
        )],
        delete_surrounding: None,
    };

    assert_eq!(
        event.validate(),
        Err(UiImePreeditClauseError::ClausesRequirePreedit)
    );
}
