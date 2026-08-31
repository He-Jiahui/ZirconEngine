---
title: Runtime99ZL Single-Pass Legacy Graph Evaluation
category: zircon_runtime
report_id: Runtime99ZL-single-pass-legacy-graph-evaluation-2026-08-28
date: 2026-08-28
session_id: root-runtime99zl-single-pass-legacy-graph-evaluation-20260828
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime99ZL Single-Pass Legacy Graph Evaluation

## Scope

The legacy animation manager previously resolved every reachable graph node by scanning the full
source node vector. Each recursive call also cloned its node ID into a visited set and returned a
new clip vector that its parent extended or mapped into another vector. A wide 4,096-clip output
therefore performed more than eight million node probes and constructed thousands of intermediate
vectors before producing the final evaluation.

Evaluation now builds one borrowed node index, retaining the first source definition for duplicate
IDs to match the previous `find_map` behavior. Recursive evaluation appends directly into one
preallocated final clip vector. Blend and additive nodes mutate only the slice appended by their
current child, while the borrowed visited set remains a recursion-stack guard and is never used as
an iteration source. The first output node, source child order, clip order, weights, target masks,
missing-node behavior, and cycle isolation remain unchanged.

## Performance Evidence

The isolated Rust model mirrors a 4,096-clip wide graph, a string-keyed source node vector, a
recursive stack guard, and final clip collection. It compares the previous per-node linear scan,
owned visited keys, and recursive vectors with the borrowed index and single output vector. Each
run uses 17 alternating sample pairs and was compiled with `rustc +1.94.1 -O -C target-cpu=native`
on Windows.

| Metric | Linear recursive path | Indexed single-pass path | Change |
|---|---:|---:|---:|
| Node probes | 8,394,753 | 4,097 | -99.951% |
| Recursive result Vec constructions | 4,097 | 0 | -100.000% |
| Copied visited-key payload | 40,966 bytes | 0 bytes | -100.000% |
| P50 | 60,355,600 ns | 1,454,900 ns | -97.589% |
| P95 | 148,349,100 ns | 8,835,500 ns | -94.044% |

The checksum remained `8,390,656` on both paths. The three post-implementation runs reduced P50
by 97.589%-98.232% and P95 by 94.044%-97.562%.

Model source:

- `.codex/state/session-coordinator/plugins13-single-pass-animation-graph-evaluation-model.rs`

The model predates the Runtime99ZL owner correction and retains its original filename and output
label. Runtime99ZL supersedes the earlier Plugins13 review for this source path. These algorithmic
and timing measurements do not replace managed Cargo behavior tests or whole-frame profiling.

## Contracts And Validation

- `tools/tests/test_runtime99zl_single_pass_legacy_graph_evaluation_performance_contract.py`
  locks first-definition borrowed indexing, one preallocated result vector, borrowed recursion keys,
  branch-local in-place weight adjustment, and removal of recursive result-vector composition.
- TDD RED failed all four contract tests against the previous implementation; the implemented
  contract passes 4/4.
- Python bytecode compilation, scoped `rustfmt +1.94.1 --edition 2021 --check`, and scoped
  `git diff --check` pass.
- Existing Rust integration coverage
  `animation_graph_evaluation_reports_additive_mask_and_clip_targets` remains the managed behavior
  gate for additive weights, masks, clip ordering, and output projection.
- Cargo type checking and the focused Rust behavior test remain pending in a managed asynchronous
  coordinator batch; no direct Cargo command was run.

## Remaining Parent-Plan Work

Runtime99ZL still owns convergence onto one compiled animation authority, stable slot identity,
resident artifacts, bounded worker scheduling, root motion, deformation correctness, and full
scale qualification. This slice only removes the quadratic lookup and recursive allocation pattern
from the legacy graph evaluator while preserving its current observable semantics.
