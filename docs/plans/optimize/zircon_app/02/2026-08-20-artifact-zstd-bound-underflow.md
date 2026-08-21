# App02 Artifact Zstd Bound Underflow Repair

- Date: 2026-08-20
- Session: `optimize-app02-artifact-zstd-bound-r1-01a00797-20260820`
- Status: focused regressions passed, containing managed batch and real-viewer gate pending

## Change

The artifact compressed-size guard now computes the sub-128 KiB Zstd margin
through `bool::then`, so the subtraction is never evaluated for larger raw
payloads. The bound remains `raw + raw / 256` plus the documented small-input
margin. Two focused tests cover exact-bound acceptance and rejection one byte
above the bound.

## Failure Evidence

Main job `06d69e9c76204ccf94fb9d79cde60ab2`, run
`093eabe96ddc4d5dbffa3c6d718265ed`, passed 5/5 WGPU timer tests, 9/9 runtime
GPU timing tests, and 128/128 viewer tests. The subsequent real viewer then
panicked at `zircon_runtime/src/asset/artifact/store.rs:619` with
`attempt to subtract with overflow`, so no screenshot or GPU timing marker was
produced and the downstream continuation stages did not run.

## Validation

Main replacement copy `39ce22ca381b43b3bbcd38b30b1c3847`, run
`498e9a342ea8469e98ad1d6f136c4508`, passed both focused artifact regressions
(`2/2`). The same containing batch later failed the real-viewer graph gate, so
the integration candidate remains pending and this record contains no
fabricated performance result. The artifact fix will remain in the next
grouped Main continuation instead of being rerun as a one-test Cargo job.
Final App01/App02/Runtime02/Runtime04 data will be sent by the coordinator's
WeCom notification for the accepted commit.
