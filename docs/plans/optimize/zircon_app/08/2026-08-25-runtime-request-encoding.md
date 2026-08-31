Plan: docs/plans/optimize/zircon_app/08-product-host-bootstrap-loop-dynamic-runtime-shutdown-current-source-review.md
Milestone: M5
Status: completed
Files: ["zircon_app/src/entry/runtime_library/runtime_session/request_encoding.rs", "zircon_app/src/entry/runtime_library/runtime_session/request_encoding/performance_tests.rs", "tools/tests/test_app08_runtime_request_encoding_performance_contract.py"]

# App08 Runtime Request Encoding

## Scope delivered

This batch reduces bounded whole-buffer JSON request encoding overhead for the App-to-Runtime ABI
without changing the serialized bytes, request limits, nesting policy, item accounting, empty
policy, or final deadline decision.

- The request writer reserves at most 4 KiB, bounded by the operation's encoded-byte limit. Common
  profile and plugin-subscription requests therefore avoid repeated `Vec` growth without reserving
  an operation's possible 256 KiB or 1 MiB maximum.
- High-resolution deadline reads are performed on the first write, after each additional 1 KiB of
  output, and at finish. Byte overflow and JSON nesting remain checked on every serializer write.
- The finish path always rechecks the deadline, so a large single serializer write cannot be
  accepted after exceeding the operation budget.
- Focused behavior coverage compares the produced bytes to `serde_json::to_vec` and records the
  actual test-build deadline checks and capacity growths.

This is one P1-13/G13 serialization slice. The broader App08 plan remains open: typed/paged control
envelopes, large-blob transfer, startup/frame/idle/shutdown/RSS/power budgets, product DLL and fault
fixtures, lifecycle and surface recovery, and full product qualification are not claimed here.

## Fresh testing evidence

TDD first produced three failures against the old zero-capacity, per-write clock-checking writer
and missing release benchmark. After implementation, the Python source contract passes 3/3,
Python bytecode compilation passes, Rust 1.94.1 formatting/parsing passes for both Rust files, and
scoped whitespace validation passes.

Five process-level repetitions of a standalone Rust 1.94.1 optimized benchmark produced these
median-of-run nearest-rank values. Each process encoded a 926-byte, 256-value request represented
by the same 513 small token writes, using 21 alternating legacy/optimized pairs and 128 encodings
per sample. The managed ignored test serializes the actual `serde::Serialize` request through the
real module writer with the same sample protocol.

| metric | legacy | optimized | reduction |
| --- | ---: | ---: | ---: |
| deadline checks per encode | 514 | 2 | 99.611% |
| `Vec` capacity growths per encode | 8 | 0 | 100% |
| encoding P50 | 29.177 us | 4.095 us | 85.965% |
| encoding P95 | 62.442 us | 6.716 us | 89.244% |

The Windows managed validation batch compiles the actual `zircon_app` library without default
features, runs byte-equivalence behavior coverage, and enforces at least a 20% P95 reduction. No
local Cargo command or Cargo dry-run was launched.

## Review

The first deadline check remains before any bytes are accepted. Between checks, the only deferred
work is at most one interval plus one serializer chunk; the unconditional finish check preserves
the terminal budget decision. The 4 KiB reservation is capped by the operation limit, so a smaller
future request policy cannot allocate beyond its own maximum. Test-only counters compile out of
product builds. Independent review remains an integration gate after managed validation returns.
