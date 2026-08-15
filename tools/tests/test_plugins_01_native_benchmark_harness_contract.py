from __future__ import annotations

import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
LOADER_ROOT = REPO_ROOT / "zircon_runtime/src/plugin/native_plugin_loader"
HARNESS_PATH = LOADER_ROOT / "benchmark_harness.rs"
CALLBACK_BENCHMARK_PATH = (
    LOADER_ROOT / "native_plugin_live_host/tests/callback_lease.rs"
)
CONTEXT_BENCHMARK_PATH = LOADER_ROOT / "host_api_adapter/context_handles/tests.rs"
REGISTRATION_BENCHMARK_PATH = (
    LOADER_ROOT / "native_plugin_live_host/tests/registration_replay.rs"
)
BROADCAST_BENCHMARK_PATH = LOADER_ROOT / "native_plugin_live_host/tests/runtime_behavior.rs"


class Plugins01NativeBenchmarkHarnessContractTests(unittest.TestCase):
    def test_harness_requires_source_profile_and_debug_markers(self) -> None:
        source = HARNESS_PATH.read_text(encoding="utf-8")

        self.assertIn("ZR_BENCHMARK_SOURCE_MANIFEST", source)
        self.assertIn("ZR_BENCHMARK_CARGO_PROFILE", source)
        self.assertIn('"release" | "profiling"', source)
        self.assertIn("cfg!(debug_assertions)", source)
        self.assertIn('"source_manifest"', source)
        self.assertIn('"debug_assertions"', source)
        self.assertIn('"zircon.native.benchmark/2"', source)
        self.assertIn(
            '\\"latency_percentile_algorithm\\":\\"nearest_rank\\"', source
        )

    def test_callback_fixture_binds_metadata_before_timing(self) -> None:
        source = CALLBACK_BENCHMARK_PATH.read_text(encoding="utf-8")
        metadata = source.index("BenchmarkRunMetadata::from_environment")
        timing = source.index("std::time::Instant::now()", metadata)

        self.assertLess(metadata, timing)
        self.assertIn("BenchmarkMeasurement", source)

    def test_every_workload_binds_metadata_before_fixture_or_timing(self) -> None:
        workloads = (
            (
                CALLBACK_BENCHMARK_PATH,
                "native_live_host_test_plugin_with_behavior(",
                "std::time::Instant::now()",
            ),
            (CONTEXT_BENCHMARK_PATH, "HostContextRegistry::default()", "Instant::now()"),
            (
                REGISTRATION_BENCHMARK_PATH,
                "replay_native_registration_scale_fixture(",
                "Instant::now()",
            ),
            (
                BROADCAST_BENCHMARK_PATH,
                "native_runtime_broadcast_benchmark_host(",
                "Instant::now()",
            ),
        )

        for path, fixture, timing in workloads:
            source = path.read_text(encoding="utf-8")
            metadata = source.index("BenchmarkRunMetadata::from_environment")
            self.assertLess(metadata, source.index(fixture, metadata))
            self.assertLess(metadata, source.index(timing, metadata))

    def test_threaded_throughput_stops_timing_before_worker_join(self) -> None:
        callback = CALLBACK_BENCHMARK_PATH.read_text(encoding="utf-8")
        context = CONTEXT_BENCHMARK_PATH.read_text(encoding="utf-8")
        callback_throughput = callback.split(
            "fn run_callback_lease_benchmark(thread_count: usize)", 1
        )[1].split("struct CallbackLeaseWorkers", 1)[0]
        context_throughput = context.split(
            "fn run_stable_lookup_benchmark(threads: usize)", 1
        )[1].split("struct StableLookupWorkers", 1)[0]

        for source in (callback_throughput, context_throughput):
            self.assertLess(
                source.index("let elapsed = started.elapsed()"),
                source.index("for worker in workers.threads"),
            )

    def test_threaded_throughput_waits_for_workers_before_timing(self) -> None:
        callback = CALLBACK_BENCHMARK_PATH.read_text(encoding="utf-8")
        context = CONTEXT_BENCHMARK_PATH.read_text(encoding="utf-8")
        callback_throughput = callback.split(
            "fn run_callback_lease_benchmark(thread_count: usize)", 1
        )[1].split("struct CallbackLeaseWorkers", 1)[0]
        context_throughput = context.split(
            "fn run_stable_lookup_benchmark(threads: usize)", 1
        )[1].split("struct StableLookupWorkers", 1)[0]

        for source in (callback_throughput, context_throughput):
            self.assertLess(
                source.index("workers.start.wait_until_ready();"),
                source.index("let started ="),
            )
            self.assertLess(
                source.index("let started ="),
                source.index("workers.start.start();"),
            )

        harness = HARNESS_PATH.read_text(encoding="utf-8")
        self.assertIn("struct BenchmarkWorkerStartGate", harness)
        self.assertIn("fn wait_until_ready(&self)", harness)
        self.assertIn("fn await_start(&self)", harness)

    def test_callback_warmup_releases_workers_after_ready_barrier(self) -> None:
        callback = CALLBACK_BENCHMARK_PATH.read_text(encoding="utf-8")
        warmup = callback.split("fn run_callback_lease_batch", 1)[1].split(
            "fn callback_lease_workers", 1
        )[0]

        self.assertIn("workers.start.wait_until_ready();", warmup)
        self.assertIn("workers.start.start();", warmup)
        ready = warmup.index("workers.start.wait_until_ready();")
        start = warmup.index("workers.start.start();")
        completion = warmup.index("workers.wait_for_completion();")

        self.assertLess(ready, start)
        self.assertLess(start, completion)

    def test_threaded_completion_gate_does_not_allocate_in_timed_interval(self) -> None:
        harness = HARNESS_PATH.read_text(encoding="utf-8")
        callback = CALLBACK_BENCHMARK_PATH.read_text(encoding="utf-8")
        context = CONTEXT_BENCHMARK_PATH.read_text(encoding="utf-8")
        callback_workers = callback.split("struct CallbackLeaseWorkers", 1)[1]
        context_workers = context.split("struct StableLookupWorkers", 1)[1].split(
            "struct StableLookupLatencySample", 1
        )[0]

        for source in (callback_workers, context_workers):
            self.assertNotIn("mpsc", source)
            self.assertIn("BenchmarkWorkerCompletionGate", source)
        completion_gate = harness.split("struct BenchmarkWorkerCompletionGate", 1)[1]
        self.assertIn("AtomicUsize", completion_gate)
        self.assertIn(".unpark()", completion_gate)

    def test_context_writer_counter_reports_measured_interval_delta(self) -> None:
        source = CONTEXT_BENCHMARK_PATH.read_text(encoding="utf-8")
        throughput = source.split(
            "fn run_stable_lookup_benchmark(threads: usize)", 1
        )[1].split("struct StableLookupWorkers", 1)[0]

        self.assertIn("saturating_sub(writer_acquires_before)", throughput)

    def test_shared_harness_keeps_each_workload_shape_in_its_own_process(self) -> None:
        harness = HARNESS_PATH.read_text(encoding="utf-8")
        callback = CALLBACK_BENCHMARK_PATH.read_text(encoding="utf-8")
        context = CONTEXT_BENCHMARK_PATH.read_text(encoding="utf-8")
        registration = REGISTRATION_BENCHMARK_PATH.read_text(encoding="utf-8")
        broadcast = BROADCAST_BENCHMARK_PATH.read_text(encoding="utf-8")

        self.assertIn('BENCHMARK_RECORD_SCHEMA', harness)
        self.assertIn('"warmup_operations"', harness)
        self.assertIn('"measured_operations"', harness)
        self.assertIn('"elapsed_ns"', harness)
        self.assertIn("latency_sample_count", harness)
        self.assertIn("latency_p95_ns", harness)
        self.assertIn("latency_sampling_ratio_numerator", harness)
        self.assertIn("latency_observer_elapsed_ns", harness)

        for source in (callback, context, registration, broadcast):
            self.assertIn("BenchmarkRunMetadata", source)

        for threads in (1, 2, 16, 64):
            self.assertIn(
                f"fn native_callback_atomic_lease_{threads}_thread_benchmark()", callback
            )
        for threads in (1, 16):
            self.assertIn(
                f"fn native_host_context_lookup_{threads}_thread_benchmark()", context
            )
        for systems in (1, 100, 1_000):
            for methods in (1, 100):
                self.assertIn(
                    f"fn native_registration_replay_{systems}_systems_{methods}_methods_benchmark()",
                    registration,
                )
        for plugins in (1, 8, 32):
            self.assertIn(
                f"fn native_runtime_broadcast_{plugins}_plugin_benchmark()", broadcast
            )

        self.assertIn(
            "const CONTEXT_BENCHMARK_MAX_LATENCY_SAMPLES: usize = 8_192", context
        )
        self.assertIn("run_stable_lookup_latency_sample(", context)
        self.assertIn("latency_sample: Some", context)

        context_throughput = context.split(
            "fn run_stable_lookup_benchmark(threads: usize)", 1
        )[1].split("fn run_stable_lookup_latency_sample(", 1)[0]
        self.assertNotIn("latencies_ns", context_throughput)
        self.assertNotIn("lookup_started", context_throughput)

        callback_core = callback.split("fn callback_lease_workers(", 1)[1]
        self.assertNotIn("Instant::now", callback_core)

        registration_core = registration.split(
            "fn run_native_registration_replay_benchmark(", 1
        )[1].split("fn replay_native_registration_scale_fixture(", 1)[0]
        self.assertEqual(registration_core.count("Instant::now()"), 1)

        broadcast_core = broadcast.split(
            "fn run_native_runtime_broadcast_benchmark(", 1
        )[1].split("fn native_runtime_broadcast_benchmark_host(", 1)[0]
        self.assertEqual(broadcast_core.count("Instant::now()"), 1)

    def test_latency_observer_elapsed_includes_post_sample_sorting(self) -> None:
        source = HARNESS_PATH.read_text(encoding="utf-8")

        finalize = re.search(
            r"fn finalize\(&mut self\) -> BenchmarkLatencySummary \{(?P<body>.*?)\n    \}",
            source,
            re.DOTALL,
        )
        self.assertIsNotNone(finalize)
        body = finalize.group("body")
        self.assertIn("self.samples_ns.sort_unstable()", body)
        self.assertIn("saturating_add(finalization_started.elapsed())", body)

    def test_latency_percentiles_use_nearest_rank_for_tail_reporting(self) -> None:
        source = HARNESS_PATH.read_text(encoding="utf-8")
        percentile = re.search(
            r"fn percentile\(sorted_samples_ns: &\[u64\], percentile: usize\) -> u64 \{(?P<body>.*?)\n\}",
            source,
            re.DOTALL,
        )
        self.assertIsNotNone(percentile)
        body = percentile.group("body")

        self.assertIn(".div_ceil(100)", body)
        self.assertNotIn("(sorted_samples_ns.len() - 1) * percentile", body)

    def test_harness_json_encodes_dynamic_metadata_and_counter_keys(self) -> None:
        source = HARNESS_PATH.read_text(encoding="utf-8")

        self.assertIn("fn json_string(value: &str) -> String", source)
        self.assertIn("json_string(self.workload)", source)
        self.assertIn("json_string(&self.shape)", source)
        self.assertIn("json_string(name)", source)
        self.assertIn("\\\\u{:04x}", source)


if __name__ == "__main__":
    unittest.main()
