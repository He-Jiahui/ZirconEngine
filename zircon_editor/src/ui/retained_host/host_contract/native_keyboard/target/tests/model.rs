use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 17;
const COMMANDS_PER_SAMPLE: usize = 2_097_152;

#[test]
fn command_palette_window_navigation_does_not_wrap_terminal_rows() {
    for total_count in [1, 12] {
        let target = target(total_count, 0, total_count - 1);
        assert!(target
            .next_move(WorkbenchPopupKeyboardCommand::Next)
            .is_none());
    }
}

#[test]
fn command_palette_window_navigation_requests_deep_pages() {
    let thirteen = target(13, 0, 11);
    assert_window_request(
        thirteen
            .next_move(WorkbenchPopupKeyboardCommand::Next)
            .expect("thirteenth command should require the next window"),
        0,
        12,
        PopupKeyboardWindowFocus::First,
    );

    let thousand = target(1_000, 0, 0);
    assert_window_request(
        thousand
            .next_move(WorkbenchPopupKeyboardCommand::Last)
            .expect("End should request the terminal command window"),
        0,
        996,
        PopupKeyboardWindowFocus::Last,
    );
}

#[test]
fn optimization_batch_ex_editor386_skips_page_math_for_in_window_commands() {
    let target = target(1_000, 0, 3);

    assert!(target
        .window_request(WorkbenchPopupKeyboardCommand::Next)
        .is_none());
    assert!(target
        .window_request(WorkbenchPopupKeyboardCommand::Previous)
        .is_none());
    assert!(target
        .window_request(WorkbenchPopupKeyboardCommand::First)
        .is_none());
    assert!(target
        .window_request(WorkbenchPopupKeyboardCommand::PageUp)
        .is_none());
    assert!(target
        .window_request(WorkbenchPopupKeyboardCommand::Accept)
        .is_none());

    let production = include_str!("../model.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("production source");
    let window_request = production
        .split("fn window_request")
        .nth(1)
        .expect("window request function");
    let early_match = window_request
        .find("match command")
        .expect("early command gate");
    let page_math = window_request
        .find("let count = self.window_count.max(1);")
        .expect("page math");
    assert!(early_match < page_math);
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_ex_editor386_in_window_keyboard_gate_benchmark() {
    let target = target(1_000, 0, 3);
    for _ in 0..4 {
        black_box(measure_legacy_common_next(&target));
        black_box(measure_fast_common_next(&target));
    }
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        if pair_index % 2 == 0 {
            legacy_samples.push(measure_legacy_common_next(&target));
            optimized_samples.push(measure_fast_common_next(&target));
        } else {
            optimized_samples.push(measure_fast_common_next(&target));
            legacy_samples.push(measure_legacy_common_next(&target));
        }
    }

    report_performance(&legacy_samples, &optimized_samples);
}

fn measure_legacy_common_next(target: &PopupKeyboardTarget) -> u128 {
    measure_common_next(target, |target| {
        let count = target.window_count.max(1);
        let last_offset = target
            .total_count
            .saturating_sub(1)
            .checked_div(count)
            .unwrap_or(0)
            .saturating_mul(count);
        target.current_index + 1 == target.rows.len() && target.window_offset < last_offset
    })
}

fn measure_fast_common_next(target: &PopupKeyboardTarget) -> u128 {
    measure_common_next(target, |target| {
        target
            .window_request(WorkbenchPopupKeyboardCommand::Next)
            .is_some()
    })
}

fn measure_common_next(
    target: &PopupKeyboardTarget,
    mut probe: impl FnMut(&PopupKeyboardTarget) -> bool,
) -> u128 {
    let started = Instant::now();
    let mut requests = 0_usize;
    for _ in 0..COMMANDS_PER_SAMPLE {
        requests += usize::from(probe(black_box(target)));
    }
    assert_eq!(black_box(requests), 0);
    started.elapsed().as_nanos().max(1)
}

fn report_performance(legacy_samples: &[u128], optimized_samples: &[u128]) {
    let legacy_p95 = nearest_rank_p95(legacy_samples);
    let optimized_p95 = nearest_rank_p95(optimized_samples);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "EDITOR386_IN_WINDOW_KEYBOARD_GATE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} commands_per_sample={COMMANDS_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=40",
        csv(legacy_samples),
        csv(optimized_samples),
    );
    assert!(
        optimized_p95 <= legacy_p95.saturating_mul(60) / 100,
        "in-window keyboard gate must reduce P95 by at least 40%"
    );
}

fn nearest_rank_p95(samples: &[u128]) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * 95).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn target(total_count: usize, window_offset: usize, current_index: usize) -> PopupKeyboardTarget {
    let row_count = total_count.saturating_sub(window_offset).min(12);
    let rows = (0..row_count).map(row).collect::<Vec<_>>();
    PopupKeyboardTarget {
        control_id: "WorkbenchCommandPalette".into(),
        dispatch_kind: "workbench_option".into(),
        current_row: rows.get(current_index).cloned(),
        current_frame: FrameRect::default(),
        popup_frame: FrameRect::default(),
        rows,
        current_index,
        window_offset,
        window_count: 12,
        total_count,
        window_navigation_enabled: true,
        window_query: "query".into(),
    }
}

fn row(index: usize) -> PopupKeyboardRow {
    PopupKeyboardRow {
        action_id: format!("command_{index}").into(),
        value_text: format!("command_{index}").into(),
        identity: format!("command_{index}").into(),
        search_text: format!("Command {index}").into(),
        focused: false,
        selected: false,
        source_index: Some(index),
        frame: FrameRect::default(),
    }
}

fn assert_window_request(
    movement: PopupKeyboardMove,
    current_offset: usize,
    target_offset: usize,
    focus: PopupKeyboardWindowFocus,
) {
    let PopupKeyboardMove::Window(request) = movement else {
        panic!("expected a command palette window request");
    };
    assert_eq!(request.current_offset, current_offset);
    assert_eq!(request.target_offset, target_offset);
    assert_eq!(request.query.as_str(), "query");
    assert!(matches!(
        (request.focus, focus),
        (
            PopupKeyboardWindowFocus::First,
            PopupKeyboardWindowFocus::First
        ) | (
            PopupKeyboardWindowFocus::Last,
            PopupKeyboardWindowFocus::Last
        )
    ));
}
