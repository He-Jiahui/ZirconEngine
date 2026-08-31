# RRT-P1-014 rich single-flight contention instrumentation

Status: `RRT-P1-014_contention_measurement_static_complete / bounded_worker_cancellation_and_managed_profile_pending`

Current source already isolates decorator panic and metadata/output budgets. The remaining provider gap is
non-cooperative deadline/cancel. The compiled cache still uses `OnceLock::get_or_init`, so a hung initializer
can block all same-key callers, but old cache telemetry could not quantify it.

This slice preserves the single-flight algorithm and adds owner-local measurement: current compile requests
in flight plus completed waiter count, total wait nanoseconds, and maximum wait nanoseconds. A call-local
initializer marker prevents the compiling caller from being counted as a waiter; an RAII guard cleans up the
gauge on return or unwind. The fixed cache profile now has 16 fields and no content/dynamic labels.

Unreal's synchronous instance-local `FShapedTextCache::FindOrAddShapedText` is an ownership reference, not a
cross-thread algorithm to copy. Per-caller duplicate parsing and arbitrary timeout are explicitly rejected
until the 1/2/4/8-caller, 1/4/16-KiB, built-in/custom/fault matrix produces wait, CPU, allocation/RSS, and power
evidence through the managed E/D/F path.

Static contracts pass 36/36 in the final 0.206 s rerun; rustfmt/diff-check pass. Production/tests/profile
owners are 541/340/739 lines. Managed Cargo/profile/fault evidence and bounded worker/cancellation remain
open. See
[`../../../zircon_runtime/text/07/2026-08-30-rich-single-flight-contention-instrumentation.md`](../../../zircon_runtime/text/07/2026-08-30-rich-single-flight-contention-instrumentation.md).
