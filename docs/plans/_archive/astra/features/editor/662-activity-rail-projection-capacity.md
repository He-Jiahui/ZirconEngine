---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/167/2026-08-26-activity-rail-profile-capacity.md
  - docs/plans/optimize/zircon_editor/01/2026-08-29-ui-hotspot-ownership-review.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
related_code:
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection/activity_rail.rs
tests:
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection/activity_rail.rs
---

# Activity Rail Projection Capacity

`expand_activity_rail_button_nodes` now reserves the conservative output upper
bound `raw_nodes.len() + 2 * tabs.row_count()` before consuming authored
templates. The bound accounts for every retained raw node plus the button and
icon projection emitted for each tab, and uses saturating arithmetic for
malformed or oversized model counts. Template dominance, tab order, selected
state, icon styling, fallback behavior, and control identifiers are unchanged.

## Plan Completion List

| Batch | Work | Status | Validation evidence |
| --- | --- | --- | --- |
| Editor167 / activity-rail projection | Reserve the raw-template and per-tab output upper bound | implemented_pending_validation | Focused Rust source regression asserts the bounded reservation and retained-node behavior. Scoped Rustfmt, diff check, source invariant probe, and the combined Runtime/Editor static-contract batch passed (`95/95`, `0.165s`). Managed Editor Cargo and Windows Release allocation/time p50/p95/p99 evidence remain pending; no coordinator state was polled in this slice. |

## Complexity Boundary

The projection remains linear in authored template nodes plus visible tab rows,
`O(raw_nodes + tabs)`. This change removes geometric output-vector growth on
the common path; it does not alter template lookup or tab traversal complexity
and does not claim product allocation or latency improvement until managed
release measurements are available.

## Static Evidence

- `rustfmt --edition 2021 --check --config skip_children=true` passed for the
  production module.
- Scoped `git diff --check` passed; Git reported only the repository's existing
  line-ending notice.
- The source probe confirmed the per-tab bound, saturating arithmetic, absence
  of the empty-vector start, and preservation of both template and tab loops.
- The combined Runtime/Editor static-contract batch passed `95/95` in `0.165s`.

## Source Snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection/activity_rail.rs` | `5CDAD98AACC67E82F6B93F96AAF0E967D60949015307F65432D170FB5E4F2109` |

## Managed Gate

No direct Cargo command or coordinator status query was run for this slice.
The next immutable multi-task validation input must compile the focused Editor
chrome projection tests with the existing Runtime/Editor batch and collect a
Windows Release allocation/time comparison. Until that batch succeeds, this
record remains `implemented_pending_validation` and makes no product
performance or p50/p95/p99 claim.
