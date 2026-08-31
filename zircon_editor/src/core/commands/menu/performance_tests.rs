use std::hint::black_box;
use std::time::Instant;

use crate::core::editor_event::{EditorEvent, EditorEventTransient};
use crate::core::editor_operation::EditorOperationPath;
use crate::core::i18n::EditorI18nService;

use super::{menu_bar_model, menu_bar_slot, MENU_ORDER};
use crate::core::commands::{
    CommandEvalCtx, EditorCommandAction, EditorCommandCategory, EditorCommandDescriptor,
    EditorCommandMenuPath, EditorCommandMenuProjection, EditorCommandRegistry,
};

const BENCH_COMMAND_COUNT: usize = 100_000;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn editor08_menu_bar_single_pass_preserves_order_filtering_and_items() {
    let mut registry = EditorCommandRegistry::default();
    for descriptor in [
        command("test.file.open", "file", &[]),
        command("test.edit.undo", "edit", &[]),
        command("test.window.reset", "window", &["layout"]),
        command("test.tools.audit", "tools", &[]),
        command("test.extension.hidden", "file", &["extension"])
            .with_menu_projection(EditorCommandMenuProjection::ExtensionRegistry),
        EditorCommandDescriptor::new(
            operation("test.palette.hidden"),
            EditorCommandCategory::Command,
            EditorCommandAction::Emit(EditorEvent::Transient(
                EditorEventTransient::OpenCommandPalette,
            )),
        )
        .with_menu_path(EditorCommandMenuPath::builtin(
            &operation("test.palette.hidden"),
            "view",
            &[],
        )),
    ] {
        registry
            .register(descriptor)
            .expect("menu benchmark command should register");
    }
    let context = CommandEvalCtx::interactive();
    let i18n = EditorI18nService::default();
    let locale = i18n.active_locale();
    let actual = menu_bar_model(&registry, &i18n, &locale, &context);

    assert_eq!(
        actual
            .menus
            .iter()
            .map(|menu| menu.label.as_str())
            .collect::<Vec<_>>(),
        ["File", "Edit", "Window", "Tools"]
    );
    assert_eq!(actual.menus[2].items[0].label, "Layout");
    assert_eq!(
        actual.menus[2].items[0].children[0].label,
        "command.test.window.reset.label"
    );
}

#[test]
#[ignore = "release-only menu-bar projection benchmark"]
fn editor08_menu_bar_single_pass_release_benchmark_evidence() {
    let paths = (0..BENCH_COMMAND_COUNT)
        .map(|index| {
            let menu = MENU_ORDER[index % MENU_ORDER.len()];
            format!("{menu}/Group {index:06}/Command {index:06}")
        })
        .collect::<Vec<_>>();
    assert_eq!(legacy_checksum(&paths), single_pass_checksum(&paths));

    let (legacy_samples, single_pass_samples) =
        paired_samples(|| measure_legacy(&paths), || measure_single_pass(&paths));
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let single_pass_p50_ns = percentile(&single_pass_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let single_pass_p95_ns = percentile(&single_pass_samples, 95);

    println!(
        "PERF_RESULT plan=Editor08 task=menu_bar_single_pass \
sample_pairs={SAMPLE_PAIRS} command_count={BENCH_COMMAND_COUNT} menu_count=7 \
legacy_projection=seven_registry_scans optimized_projection=single_registry_scan_fixed_buckets \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={single_pass_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={single_pass_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        raw(&legacy_samples),
        raw(&single_pass_samples),
    );

    assert!(
        single_pass_p95_ns.saturating_mul(10) <= legacy_p95_ns.saturating_mul(3),
        "single-pass menu projection must reduce P95 by at least 70%: \
legacy={legacy_p95_ns}ns optimized={single_pass_p95_ns}ns"
    );
}

fn command(id: &str, root: &str, groups: &[&str]) -> EditorCommandDescriptor {
    let operation = operation(id);
    EditorCommandDescriptor::new(
        operation.clone(),
        EditorCommandCategory::Command,
        EditorCommandAction::Operation,
    )
    .with_menu_path(EditorCommandMenuPath::builtin(&operation, root, groups))
}

fn operation(value: &str) -> EditorOperationPath {
    EditorOperationPath::parse(value).expect("valid benchmark operation path")
}

fn legacy_checksum(paths: &[String]) -> usize {
    MENU_ORDER
        .iter()
        .enumerate()
        .map(|(slot, menu)| {
            paths
                .iter()
                .enumerate()
                .filter_map(|(index, path)| {
                    let (top_level, _) = path.split_once('/')?;
                    (top_level == *menu).then_some(index ^ slot)
                })
                .fold(0usize, usize::wrapping_add)
        })
        .fold(0usize, usize::wrapping_add)
}

fn single_pass_checksum(paths: &[String]) -> usize {
    paths
        .iter()
        .enumerate()
        .filter_map(|(index, path)| {
            let (top_level, _) = path.split_once('/')?;
            menu_bar_slot(top_level).map(|slot| index ^ slot)
        })
        .fold(0usize, usize::wrapping_add)
}

fn paired_samples(
    mut measure_legacy: impl FnMut() -> u128,
    mut measure_single_pass: impl FnMut() -> u128,
) -> (Vec<u128>, Vec<u128>) {
    for _ in 0..4 {
        black_box(measure_legacy());
        black_box(measure_single_pass());
    }
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut single_pass_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy());
            single_pass_samples.push(measure_single_pass());
        } else {
            single_pass_samples.push(measure_single_pass());
            legacy_samples.push(measure_legacy());
        }
    }
    (legacy_samples, single_pass_samples)
}

fn measure_legacy(paths: &[String]) -> u128 {
    let started = Instant::now();
    black_box(legacy_checksum(black_box(paths)));
    started.elapsed().as_nanos().max(1)
}

fn measure_single_pass(paths: &[String]) -> u128 {
    let started = Instant::now();
    black_box(single_pass_checksum(black_box(paths)));
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn raw(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
