use std::ffi::OsString;
use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::project::RelPath;

use super::PlayProcessCommand;

const RENDERS_PER_SAMPLE: usize = 4_096;
const SAMPLE_PAIRS: usize = 101;

#[test]
fn editor886_play_process_borrowed_arguments_preserve_command_and_diagnostic() {
    let command = fixture_command();
    let expected = [
        "--project",
        ".",
        "--runtime-session-profile",
        "runtime",
        "--play-scene",
        ".zircon/play/42/play-scene.zrscene.json",
        "--play-report-pipe",
        "zircon-play-report-42",
    ];

    assert_eq!(command.arguments(), expected);
    assert_eq!(
        command.arguments().join(" "),
        "--project . --runtime-session-profile runtime --play-scene .zircon/play/42/play-scene.zrscene.json --play-report-pipe zircon-play-report-42"
    );
    assert_eq!(
        command
            .configure()
            .get_args()
            .map(|argument| argument.to_string_lossy().into_owned())
            .collect::<Vec<_>>(),
        expected.map(str::to_string)
    );
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn editor886_play_process_borrowed_arguments_benchmark() {
    let command = fixture_command();
    assert_eq!(
        optimized_argument_diagnostic(&command),
        legacy_argument_diagnostic(&command)
    );

    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(&command, legacy_argument_diagnostic));
            optimized.push(measure(&command, optimized_argument_diagnostic));
        } else {
            optimized.push(measure(&command, optimized_argument_diagnostic));
            legacy.push(measure(&command, legacy_argument_diagnostic));
        }
    }

    let legacy_p50 = percentile(&legacy, 50);
    let legacy_p95 = percentile(&legacy, 95);
    let legacy_p99 = percentile(&legacy, 99);
    let optimized_p50 = percentile(&optimized, 50);
    let optimized_p95 = percentile(&optimized, 95);
    let optimized_p99 = percentile(&optimized, 99);
    println!(
        "EDITOR886_PLAY_PROCESS_BORROWED_ARGUMENTS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} renders_per_sample={RENDERS_PER_SAMPLE} arguments_per_render=8 legacy_owned_argument_strings_per_sample=32768 legacy_temporary_vector_slots_per_sample=65536 optimized_owned_argument_strings_per_sample=0 optimized_temporary_vector_slots_per_sample=0 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} legacy_p99_ns={legacy_p99} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} optimized_p99_ns={optimized_p99}"
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(110),
        "borrowed argument join P95 {optimized_p95}ns must stay within 10% of owned argument staging P95 {legacy_p95}ns"
    );
}

fn fixture_command() -> PlayProcessCommand {
    PlayProcessCommand::new(
        "zircon_runtime",
        "project",
        RelPath::parse(".zircon/play/42/play-scene.zrscene.json")
            .expect("valid play snapshot relative path"),
        "zircon-play-report-42",
    )
}

fn legacy_argument_diagnostic(command: &PlayProcessCommand) -> String {
    command
        .arguments()
        .into_iter()
        .map(OsString::from)
        .collect::<Vec<_>>()
        .into_iter()
        .map(|argument| argument.to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join(" ")
}

fn optimized_argument_diagnostic(command: &PlayProcessCommand) -> String {
    command.arguments().join(" ")
}

fn measure(command: &PlayProcessCommand, render: fn(&PlayProcessCommand) -> String) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..RENDERS_PER_SAMPLE {
        checksum ^= black_box(render(black_box(command))).len();
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
