# Zircon App04 keybind reverse index

## Scope

- Owner report: `docs/plans/optimize/zircon_app/04-woc-native-client-window-input-shell-ui-presentation-frame-product-integration-review.md`
- Finding: `WOC-CLIENT-P1-034`
- Baseline: `a8eca85cc83008aeb200dce2d2b01e2ae3c157c9`, epoch `436`
- Current Session: `root-app04-keybind-reverse-index-release-r2-20260831`
- Reclaimed from archived Session: `root-zircon-app04-keybind-reverse-index-20260825`
- Production: `examples/woc/native/apps/woc_client/src/input/keybind/bindings.rs`
- Behavior regression: `examples/woc/native/apps/woc_client/tests/input/keybind/bindings.rs`
- Structural contract: `tools/tests/test_woc_client_keybind_reverse_index_performance_contract.py`

## Problem

`action_for_combo`, `edge_action_for_combo`, and `held_action_for_code` each walked the 61-action registry and both persisted binding slots for every dispatch. Keyboard movement invoked the Held lookup for every caller-provided physical code, multiplying that repeated string scan within the input frame. The hot path did not allocate, but its work scaled with both registry size and held-key count instead of the queried key.

## Change

- Build exact-combo and Held physical-code reverse indexes when `Keybinds` is constructed.
- Store action indexes rather than cloning action IDs; hot lookups borrow the caller's `&str` directly through `HashMap::get`.
- Preserve the original registry-order first-match behavior independently for any-kind, Edge, and Held dispatch. In particular, default `KeyA` remains Held `turnLeft`, Edge `attackMove`, and any-kind `turnLeft`.
- Rebuild the small indexes after the low-frequency `bind`, `clear`, and `reset` mutations so persisted slots remain the only source of truth.
- Keep the wider per-device transition-state and focus/release lifecycle work outside this narrow optimization; those require the input-service boundary described by the parent plan.

## TDD and static evidence

- RED: `python -m unittest tools.tests.test_woc_client_keybind_reverse_index_performance_contract -v` reported four failing contracts and eight failure points against the repeated scans.
- GREEN: the same command passes `4/4` after the reverse-index implementation.
- The actual `combo.rs`, `registry.rs`, and edited `bindings.rs` compiled together with optimized `rustc`; the wrapper then passed the shared `KeyA` bind/clear/reset scenario.
- `python -m py_compile tools/tests/test_woc_client_keybind_reverse_index_performance_contract.py` passes.
- `rustfmt +1.94.1 --edition 2021 --config skip_children=true --check` passes for both owned Rust files.
- `git diff --check` passes for the candidate paths apart from Git's existing LF/CRLF checkout notice.
- The production file is 294 lines after the change.

## Local release-model evidence

The independent Rust `-O` model uses 61 actions, up to two bindings per action, 40,000 mixed any-kind/Edge/Held queries per sample, 21 alternating legacy/indexed sample pairs, and nearest-rank percentiles.

| Metric | Linear scan | Reverse index | Change |
|---|---:|---:|---:|
| P50 | 10,834,700 ns | 1,561,000 ns | -85.588% |
| P95 | 18,446,000 ns | 2,639,400 ns | -85.691% |

The formal in-crate release benchmark repeats the same workload against the actual
`Keybinds` implementation. It first verifies indexed/legacy parity across all 61 actions
and all three lookup kinds, performs four warm-up pairs, then emits both raw 21-sample
arrays plus computed nearest-rank P50/P95 values. The Rust test itself requires at least
75% improvement for both distributions; the validator independently recomputes them
from the raw arrays.

## Async validation

No Cargo command is run directly in the shared checkout. One coordinator batch contains:

1. the four Python source contracts;
2. formatting and candidate diff checks;
3. all nine `keybind::bindings` integration regressions, including shared precedence after mutation;
4. the ignored release benchmark with `--nocapture`, including its in-test P50/P95 gates;
5. external parsing of the raw arrays and independent percentile recomputation.

The candidate remains pending until the coordinator reports both managed Cargo groups green. The historical parent-plan evidence recorded six `woc_protocol` compile errors; if they remain, this batch must report that lowest-layer failure rather than claiming an upper-layer pass. Commit and automatic WeCom finalization must quote the managed benchmark row, not promote the standalone model to crate-level acceptance.
