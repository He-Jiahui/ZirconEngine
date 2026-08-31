---
record_kind: accepted_structure_sync
status: accepted
accepted_at: 2026-07-16
status_anchor: runtime_10_runtime_api_owner_topology_8_of_8_static_accepted_cargo_unchanged
origin_plan: docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
related_code:
  - zircon_runtime_interface/src/runtime_api/mod.rs
  - zircon_runtime_interface/src/runtime_api/session/operation.rs
  - zircon_runtime_interface/src/runtime_api/session/plugin_event_mirror.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_api_boundary.py
tests:
  - python -m unittest tools.tests.test_runtime_api_boundary
  - python -m py_compile .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_api_boundary.py tools/tests/test_runtime_api_boundary.py
  - direct runtime_api_boundary_audit
---

# Runtime10 Runtime API owner topology sync

## Accepted scope

The committed V2 ABI surface has eight folder-backed owner modules. `operation` owns the generic operation handle, submit/progress/result DTOs, terminal phases, and submit/poll/harvest function types. `plugin_event_mirror` separately owns plugin-event subscription and delivery DTOs. They are distinct ABI domains and remain separate narrow files; neither is folded into the facade or an unrelated owner.

This record accepts only the Runtime10 structural-audit synchronization. It does not promote the parent plan beyond `in_progress` and does not close its pending dynamic API, app-loader, or UI/editor Cargo gates.

## Exact implementation

- `runtime_api_boundary.py` now lists the same eight owner modules already declared and re-exported by `runtime_api.rs`.
- `test_runtime_api_boundary.py` locks the exact eight-domain tuple, folder contents, facade declarations/re-exports, line budgets, and zero-risk result.
- `docs/zircon_runtime_interface/runtime_api.md` records the focused regression test alongside the existing ABI validation contract.

## Validation evidence

- Focused unittest: `1 passed`.
- Python compilation: passed for the audit owner and focused regression test.
- Direct audit: `expected_module_count = 8`, actual modules `8`, `unexpected_modules = []`, `risks = []`.
- Exact diff check: passed; only line-ending preservation warnings were emitted for existing tracked files.

## Remaining work

- Runtime10 package-level Cargo gates remain governed by the parent plan and must use a coordinator-managed Windows lane.
- Runtime02 root-seat and Runtime14 navigation topology decisions are intentionally excluded from this record.
