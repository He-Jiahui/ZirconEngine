---
title: Editor08 Palette Fixture and Capacity Completion List
category: astra-editor
report_id: Editor08-palette-fixtures-real-posting-capacity-2026-10-13
date: 2026-10-13
implementation_status: source_reviewed
validation_status: native_and_managed_pending
---

# Editor08 Palette Fixture and Capacity Completion List

## Source work reviewed

- Correct the palette catalog source slice while preserving both positive reservation assertions and both negative `Vec::new()` guards.
- Repair the MRU regression and benchmark fixtures to use canonical three-segment operation IDs; preserve missing-command omission and index membership behavior.
- Add test-only observation of real locale posting capacities and push growth. Compare actual optimized postings to a `Vec::new()` legacy collector fed the exact same normalized documents.
- Keep the V1 4,096-by-256 geometric model and its 2,816-event assertion, clearly labeled as historical synthetic workload. Add the independent V2 valid-UTF-8 actual-build producer without a timing or P95 claim.
- Preserve the existing Editor629 32,768-descriptor/17-pair/85%-of-legacy-P95 gate and the MRU 100,000-ID/32-entry/21-pair/50%-of-legacy-P95 gate.
- Preserve the real `zanzibar` query result and its one-visit/one-enablement assertions; retain valid Unicode fixture keywords and existing Unicode search regressions.
- Review the five frozen source identities and exact artifacts. The exact guard passed with zero errors and one warning for the newly ignored V2 evidence case.

## Acceptance still pending

- Run the three normal Rust filters and four ignored Rust evidence filters listed in [the detailed Editor08 record](../../../../plans/optimize/zircon_editor/08/2026-10-13-palette-fixtures-and-real-posting-capacity-record.md). Ignored runs require `--exact --ignored --nocapture --test-threads=1`.
- Run the four Python contract cases from `tools/tests/test_editor_palette_locale_posting_capacity_performance_contract.py`.
- Collect native Windows results and actual raw P95 evidence for the two preserved release gates. The actual-build V2 producer measures deterministic capacity and growth transitions only; it does not establish CPU or P95 performance.
- A separate root-side handoff reconciliation reported the canonical batch09 intake missing. This review did not recheck it and does not infer cause or queue acceptance. The batch10 handoff remains pending.

## Evidence

- Final patch: `.codex/outbox/reports/optimize-20261013-01a13451/editor-palette-contract.patch` (SHA-256 `528d29d2e42c3f699f5877baced4c81cf5ddfe0ef781ee1d4a28b9f900b44194`).
- Source hashes, filters, and static checks: `.codex/outbox/reports/optimize-20261013-01a13451/editor-palette-contract-evidence.json` (SHA-256 `3ce82f031dabfadf63bcbc991fab5d295c829973a3c04551bad243d090e14798`).
- Exact guard: `.codex/outbox/reports/optimize-20261013-01a13451/editor-palette-contract-guard.json` (SHA-256 `72a4c13e0a8fe91edcabfcea2f0651b7a5597686d755cd9bdb4b119baf38bd3e`; passed, 0 errors, 1 warning for the ignored V2 case).
- The existing ignored Python contract was edited from its verified working-tree baseline, not added as a tracked file. Baseline SHA-256: `5d57e20955fa885f05504cd4941e48327469aa1adae87388e5b0b85e3733ae42`.
- Static Rust formatting, scoped whitespace checks, and Python AST parsing passed. Cargo/native tests, Python unittest execution, and ignored performance producers have not run.
