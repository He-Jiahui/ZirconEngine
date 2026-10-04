---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/01/2026-08-29-ui-hotspot-ownership-review.md
related_code:
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection/menu_chrome.rs
tests:
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection/menu_chrome.rs
---

# Menu Chrome Projection Capacity

The menu-chrome slot expansion now reserves the raw template count plus the
maximum authored menu-slot count before projecting dynamic menu rows. The
reservation is a lower bound: missing templates and invalid rows still follow
the existing skip/fallback behavior, while menu labels, geometry, ordering,
and control identities remain unchanged.

## Plan Completion List

| Batch | Work | Status | Validation evidence |
| --- | --- | --- | --- |
| Editor01 / chrome projection | Reserve raw-node plus dynamic-slot output capacity | implemented_pending_validation | Focused source regression locks the bounded capacity expression. Scoped Rustfmt and diff checks pass; the Runtime + Editor performance-contract discovery passed `1727/1727`. Managed Editor Cargo and product pane CPU/allocation/p50/p95/p99 evidence remain pending. |

## Complexity Boundary

Projection remains `O(raw_nodes + menu_rows)` and still performs the same
template selection and per-row text/geometry work. This change only removes
geometric growth for the known lower bound; it does not claim a fixed product
latency or memory reduction.

## Source Snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection/menu_chrome.rs` | `6F64918E022F77A00B7E224760F78FC3DF126D2CCC1406E2EF852D023DF12B64` |

## Managed Gate

The attempted Cargo submission was rejected at admission by the external
`E:\\Git\\zr_vm` dirty-worktree gate after the current Session's scoped lease
and attribution were established. A separate static Runtime text ticket was
accepted asynchronously, but it does not compile this Editor production path;
no Cargo process or status polling was started. This Editor record therefore
remains `implemented_pending_validation` until an owner-attributed multi-task
ticket can compile the relevant Runtime/Editor paths and collect current-source
release evidence.
