---
related_code:
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/input_stack_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/input_stack_anchor_inventory.py
  - tools/tests/test_runtime_input_stack_audit.py
  - zircon_runtime/src/tests/runtime_absorption/input_stack/inventory/mirror_docs.rs
  - zircon_runtime/src/tests/runtime_absorption/input_stack/action_mapping.rs
  - docs/zircon_runtime/input/input_state.md
  - docs/plans/zircon_runtime/runtime/12/2026-07-17-m5-input-event-bounds-current-source-closeout.md
implementation_files:
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/input_stack_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/input_stack_anchor_inventory.py
  - tools/tests/test_runtime_input_stack_audit.py
  - zircon_runtime/src/tests/runtime_absorption/input_stack/inventory/mirror_docs.rs
  - zircon_runtime/src/tests/runtime_absorption/input_stack/action_mapping.rs
  - docs/zircon_runtime/input/input_state.md
  - docs/plans/zircon_runtime/runtime/12/2026-07-17-m5-input-event-bounds-current-source-closeout.md
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
  - docs/plans/zircon_runtime/runtime/12/2026-07-17-m5-input-event-bounds-current-source-closeout.md
tests:
  - python -m unittest tools.tests.test_runtime_input_stack_audit
  - cargo check -p zircon_runtime --lib --locked --jobs 1
  - managed Windows job d064840b0a8f40dcb405bab74b493ba1 / run 78454dffc1744c858bad697721992c7e
  - managed Windows job 586f1f84cf814180a1bc71c48a713a90 / run a101c9a710634fa386a5f50fb7f3b475
  - managed Windows job 8fbd021bed7641d3909cb981c55d083d / run 85db1f8c7a524bbe950e4ea37b65dd31
  - managed Windows job de0a56b9b2d948c9af25f5d6521cade3 / run de879c0668c04fbdac24250402ba1a2a
  - managed Windows job f6841642e70c4a43b8674c92f9f18461 / run 230eaecd12ce4bfe97d92753efff6cdc
  - managed Windows job c5d6303ce4334f3995b2b5073af7569b / run ba6b0bc8406540f586b4e2f5df7be176
  - diagnostic-only Windows job 6354ba6b1cf64c7db0e77651c3d36011 / run 49b5806d79034e5ebf91de96e12c90ce
  - diagnostic-only Windows job b683460bdf5045908517e328b85f962b / run c96ead9118594183a538a77ea88626ec
  - diagnostic-only Windows job 9d73f359158d442184b7975ee3bec370 / run f1ea7588172f4c869511fc0f14cf96c1
  - diagnostic-only Windows job 693431b2f19c45cf9d9e7d98b1032568 / run de368d8c08c24c7780e2c76b6d92c577
  - diagnostic-only Windows job 0a71bc4d34c24ccf88fc890f4b84db62 / run df0d7cc83c8541b3ac72b6aaac6977d8
doc_type: milestone-detail
---

# Runtime12 M4 Mirror-Doc Guard Post-Commit Fix

Plan: docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md

Milestone: M4

Status: waiting_prerequisite_owner_commits_and_re_review

Files: [".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/input_stack_source_inventory.py", ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/input_stack_anchor_inventory.py", "tools/tests/test_runtime_input_stack_audit.py", "zircon_runtime/src/tests/runtime_absorption/input_stack/inventory/mirror_docs.rs", "zircon_runtime/src/tests/runtime_absorption/input_stack/action_mapping.rs", "docs/zircon_runtime/input/input_state.md", "docs/plans/zircon_runtime/runtime/12/2026-07-17-m5-input-event-bounds-current-source-closeout.md", "docs/plans/zircon_runtime/runtime/12/2026-07-17-m4-mirror-doc-guard-post-commit-fix.md"]

Validated Source Manifest Files: current input behavior job `d064840b0a8f40dcb405bab74b493ba1` passed 39/39; plan-status job `586f1f84cf814180a1bc71c48a713a90` passed exactly 1/1; focused mirror reservation `14775513ea224778940f9955b57e352a` bound 57 paths at fingerprint `3c90bfb6e6e9dec5d7557da7c47b7e26f2c11a61609d2b7f33a3ac26b46dd2f5` and passed exactly 1/1; canonical check reservation `6dd22367dd5041b88ad27c024ceb07ec` bound 630 paths at fingerprint `42216a3cdd88bbed30369bcce67ad5698effc2dfccfca6f08722e22ce27b1e44` and released exit 0 with 630/630 post-run hashes matching. Action-guard reservation `5e36821bbf46414a8047ddfc3d0cfff9` bound 641 paths and passed exactly 1/1, but final exact-8 acceptance remains blocked until the uncommitted Frameworks05 module-identity inputs and Runtime12 evaluator prerequisite have immutable owner SHAs.

Date: 2026-07-17

## Scope Delivered

Runtime12 bounded input-event retention and indexed action evaluation is already committed as `94da2b39e79722a030b5aeb27fbcdbf3f2611c27`. Its managed Windows input gate passed 39/39, its canonical plan-status guard passed exactly 1/1, the direct structure audit reported runtime/framework/test `18/25/7` and behavior anchors `21` with empty unexpected/missing/wiring/risk lists, and independent review reported Critical 0 / Important 0.

## Post-commit finding and correction

The post-commit read-only review found that `runtime_12_input_stack_mirror_docs_match_structure_audit_counts` required the concise M4 closeout to duplicate every detailed audit anchor. That made the guard enforce a false mirror relationship: the module document is the detailed audit authority, while the M4 addendum is intentionally a concise accepted summary.

The correction keeps the detailed exact-anchor and uniqueness checks on `docs/zircon_runtime/input/input_state.md`, limits the M4 addendum check to its title, milestone, `18/26/7`, behavior-anchor `21`, and empty-list acceptance summary, and states that the protected parent plan/runtime index are outside this bounded business manifest. The historical filename segment `m5` is documented as an execution-batch label, not a new protected-plan milestone. No compatibility shim, skipped guard, cfg gate, or threshold weakening was introduced.

## Fresh Testing Evidence

Source-manifest-bound Windows job `591948392cbb428487ff2f3908754c36` / run `6d9bc0aa6bb243d992c23988d6e35f97` ran `cargo check -p zircon_runtime --lib --locked --jobs 1` to natural completion. It exited `0`, released with `live_process_pids = []`, and finished in 11m07s. The 511 emitted warnings are existing unused-item diagnostics and contain no Runtime12 compile error.

`python -m unittest tools.tests.test_runtime_input_stack_audit` passes 1/1, direct audit reports runtime/framework/test `18/26/7`, public surface `31/31`, behavior anchors `21`, and empty missing/unexpected/risk lists. Scoped `git diff --check` reports no whitespace error (only repository LF/CRLF notices). Independent review of the expanded correction reported Critical 0 / Important 1 / Minor 2: the canonical Rust action-mapping guard still named the removed dual-pass helpers, the Python test did not pin `31/31`, and the failure state understated completed non-Cargo evidence. All three findings are included in the current correction; final review remains pending after managed Rust validation.

The current-source input behavior gate used job `d064840b0a8f40dcb405bab74b493ba1` / run `78454dffc1744c858bad697721992c7e` and the exact command `cargo test -p zircon_runtime --lib input::tests:: --locked --jobs 1 -- --nocapture --test-threads=1`. It released with exit `0`, no live PIDs, and raw stdout proves `39 passed; 0 failed; 0 ignored; 8202 filtered` in 0.37s after a 16m05s build. This validates the bounded input stack behavior against its bound source manifest; the separate plan-status and mirror-doc guards remain responsible for closeout governance.

The source-manifest-bound focused Rust guard used reservation `0e6fda4fdfc84ae2a60e8e9565c5cce1`, source fingerprint `28beda781f8d537d2ff80f302b322106663e043c35ee2bf27c13c4576eac4dac`, job `8fbd021bed7641d3909cb981c55d083d`, and run `85db1f8c7a524bbe950e4ea37b65dd31`. Raw stdout proves exactly one test executed: `tests::runtime_absorption::input_stack::inventory::mirror_docs::runtime_12_input_stack_mirror_docs_match_structure_audit_counts ... ok`; the result is `1 passed; 0 failed; 0 ignored; 8240 filtered` in 0.01s. The isolated lib-test build finished in 42m01s, exited `0`, and released with `live_process_pids = []`. This remains valid evidence for the guard split, but the coordinator workflow then required the historical closeout field to become `Accepted Milestone: M4` so that this follow-up record is the unique current M4 manifest owner. Because that closeout is an `include_str!` input, this record returns to `validating` until the focused guard is rerun against the new source hash.

The canonical plan-status guard used job `586f1f84cf814180a1bc71c48a713a90` / run `a101c9a710634fa386a5f50fb7f3b475` and the exact command `cargo +1.94.1 test -p zircon_runtime --lib runtime_12_input_stack_cargo_pending_gate_stays_explicit_until_input_validation --locked --jobs 1 -- --nocapture --test-threads=1`. It released with exit `0`, no live PIDs, and raw stdout proves exactly one target test passed with `0 failed / 8240 filtered` in 0.10s after a 7m09s build. This is valid directional evidence that the Cargo-pending state remains explicit; it does not promote the M4 closeout by itself.

Warm reservation `c5d0693271ec432fb020c2968ca3bb0f` later produced job `6354ba6b1cf64c7db0e77651c3d36011` / run `49b5806d79034e5ebf91de96e12c90ce`. Its raw stdout contains one passing mirror-doc target (`1 passed / 0 failed / 8332 filtered`, 0.01s) and the process released after 46m47s with exit `0` and no live PIDs. However, the reservation auto-started before the Editor10 source freeze and its owner marked the run mixed-source/source-polluted. It is retained only as diagnostic evidence and cannot close Runtime12. A fresh current-source mirror-doc guard and canonical `cargo check -p zircon_runtime --lib --locked --jobs 1` remain required after Coordinator01 returns the open reservation-dependency-barrier failure.

Fresh three-input reservation `e367ef89574d4410b0b27c76212a0de2` produced job `b683460bdf5045908517e328b85f962b` / run `c96ead9118594183a538a77ea88626ec`. The job released naturally with exit `101`, `live_process_pids = []`, and never executed the mirror target. Raw stderr proves five lower lib-test compile errors: three Text01 `GlyphAtlasPageResidencyDecision` resolution errors from a source state before Text01 snapshot 497, plus two Frameworks04 E0063 errors where native live-host fixtures omitted new `missing_required_capabilities` and `denied_capabilities` fields. Text01 restored the import on its stable snapshot; the exact Frameworks04 test consumer now initializes both empty capability lists in both success fixtures. The origin failure and fixed return are recorded under Runtime12/Frameworks04. Job b683 is diagnostic-only and is not mirror red or green.

Replacement reservation `20d79ef1931b41c5812136de077abeaf` bound the three mirror inputs plus Text01 `page.rs`/`page_residency.rs` and Frameworks04 `native_plugin_abi.rs`/native live-host tests. Its source fingerprint was `f91239cfa06627fd8142bd706b48f7a09729c0a5dc712b9e9208ed132960bd8b`. It produced job `9d73f359158d442184b7975ee3bec370` / run `f1ea7588172f4c869511fc0f14cf96c1`, which released naturally with exit `101`, `live_process_pids = []`, and empty stdout before the target executed. Raw stderr records eight lower compile errors: Runtime10 V3 convergence owns the unresolved wake callback import, a non-`Send` registry test return, and two linked-plugin tick calls missing the V3 demand output pointer; the picking framework owner has four `collect` inference errors that leave an unsized slice and borrowed final hit. This run is diagnostic-only and supplies no mirror red/green claim. A new source-bound focused run is required after both lower owners return their exact fixes.

Both lower owners then closed the exact compile inputs without absorbing Runtime12 behavior. Runtime10 removed the stale callback alias, kept `ZrStatus` local to its worker threads, passed the V3 demand output pointer in linked-plugin tests, and received final independent review Critical 0 / Important 0 / Minor 0. Performance01 restored the explicit owned `Vec<IndexedHit>` collection type without changing hit sorting, cloning, grouping, or blocking semantics; its failure remains open until a managed compile proves the fix. Replacement reservation `cebbc95fdb2749f69e75db1e0678436a` initially bound 40 current-source paths with source fingerprint `dba44953b5b98dbd1e6c5bcbc558ac6147096d33ca9501c7031904b9e8b8c84c`, but a required non-Cargo audit found legitimate newer Frameworks05 module-identity and Performance01 one-pass evaluator drift before the reservation was consumed. The reservation was released unstarted: it has no Cargo job, no target result, and cannot be cited as red or green. The synchronized current source requires a new source-bound focused run.

Current-source focused mirror reservation `14775513ea224778940f9955b57e352a` bound 57 paths with source fingerprint `3c90bfb6e6e9dec5d7557da7c47b7e26f2c11a61609d2b7f33a3ac26b46dd2f5`. Job `de0a56b9b2d948c9af25f5d6521cade3` / run `de879c0668c04fbdac24250402ba1a2a` executed the exact mirror target, released with exit `0` and no live PIDs, and raw stdout proves exactly one test passed with `0 failed / 8473 filtered` in 0.03s after a 64m00s build. This is the current authoritative mirror-doc guard evidence.

Canonical check reservation `577a3a45edf748b39808324c7c317184` then bound 537 current source paths with fingerprint `db357970854aefd7c7e0680243a901ea91f9f4973bb7002742d49524ed0be2dd`. Job `693431b2f19c45cf9d9e7d98b1032568` / run `de368d8c08c24c7780e2c76b6d92c577` naturally released with exit `101`, no live PIDs, and one E0277 in the foreign Render17 path `zircon_runtime/src/rhi_wgpu/ui_surface.rs`: an already borrowed `&str` was passed to `HashMap<String, _>::get` as `&&str`. Runtime12's nine changed paths produced no compiler error. This run is diagnostic-only; it cannot close the canonical check and is not a Runtime12 guard red/green claim. Render17 owns the lowest repair and Runtime12 must rerun the action guard/check against the returned source.

After Render17 changed the lookup to `.get(cache_key)`, action-guard reservation `4d7559132f484e8daa827016a470515d` bound 542 paths with fingerprint `fc0e8cc5726aada5b310369e6948824977636c10ce20395fd1966862a60a261d`. The owner then ran rustfmt before learning that the reservation had started, moving `ui_surface.rs` from the bound `1F9E...` hash to its frozen final `D1FA...` hash. Job `0a71bc4d34c24ccf88fc890f4b84db62` / run `df0d7cc83c8541b3ac72b6aaac6977d8` was therefore source-polluted and diagnostic-only regardless of its result. It naturally released with exit `101`, no live PIDs, and never executed the target. The corrected `ui_surface` E0277 no longer appeared; compilation instead found two lower Render01 E0308 errors in `graph_execution/materialization_validation.rs`, where callers passed `&&str` to the typed `RenderGraphResource` lookup instead of using the compiled graph's name-indexed lookup. Render01 owns that lowest repair. This job cannot be cited as action-guard red or green.

Render01 then returned the typed name-lookup repair. Current-source canonical job `f6841642e70c4a43b8674c92f9f18461` / run `230eaecd12ce4bfe97d92753efff6cdc` executed `cargo +1.94.1 check -p zircon_runtime --lib --locked --jobs 1`, released with exit `0` and no live PIDs after 21.53 seconds, and matched all 630 bound hashes with no new relevant dirty path outside its manifest. This closes the Render01/Render17 compile blockers without absorbing either owner; only the corrected Runtime12 action-mapping guard and final independent review remain.

Action-guard reservation `5e36821bbf46414a8047ddfc3d0cfff9` produced job `c5d6303ce4334f3995b2b5073af7569b` / run `ba6b0bc8406540f586b4e2f5df7be176`. It executed `cargo +1.94.1 test -p zircon_runtime --lib runtime_12_action_mapping_keeps_ui_filtered_evaluation_path --locked --jobs 1 -- --nocapture --test-threads=1`, released with exit `0` and no live PIDs, and raw stdout proves exactly one target passed with `0 failed / 8534 filtered` in 0.01s after a 38m17s build. The action guard, input module doc, and M4 addendum hashes used by the target remained unchanged through review. This proves the synchronized combined source behavior, but it cannot make the exact-8 commit self-contained while its clean-HEAD prerequisites are still absent.

## Review

The current exact-8 review reports Critical 0 / Important 2 / Minor 1. It confirms all terminal counts and current combined-tree semantics, but rejects direct acceptance because Frameworks05's `module_identity.rs` owner and Runtime12's one-pass evaluator/test are not yet immutable in HEAD, the Rust guard did not forbid restoration of the old dual traversal, and the M5 review wording did not distinguish its historical snapshot. The guard now rejects the old helper definitions/calls, requires one compiled-context lookup and one `evaluate_binding_axes` owner call, and the M5 wording is scoped to the historical accepted snapshot. Final independent re-review remains mandatory after both prerequisite owner SHAs exist. No compatibility path, threshold weakening, skipped guard, or unrelated scope expansion is authorized while waiting.

The temporary `current-input-structure-audit-drift` note described work owned and repaired inside Runtime12 itself. It was therefore not a valid cross-plan failure handoff: returning it would preserve a forbidden self-edge. Its failure, root-cause, correction, and validation evidence are consolidated in this M4 record, and the invalid lifecycle artifact is retired before commit.

## 2026-07-19 final handoff audit

The canonical plan-status job `586f1f84cf814180a1bc71c48a713a90` / run `a101c9a710634fa386a5f50fb7f3b475` remains valid directional evidence: it released with exit `0`, no live PIDs, and exactly one passing `runtime_12_input_stack_cargo_pending_gate_stays_explicit_until_input_validation` target. It does not make the exact-8 manifest commit-ready.

Final independent read-only review reports Critical 0 / Important 3 / Minor 0 and rejects the milestone commit. Clean HEAD `9cbc07ca2316f752b05dbef95ade9d70e893afeb` still lacks Frameworks05's `zircon_runtime/src/core/framework/input/module_identity.rs` owner and still contains the pre-convergence action evaluator rather than the single `evaluate_binding_axes` owner required by the current guard. The Frameworks05 module-identity batch and Runtime12 one-pass evaluator/test batch therefore still require immutable owner SHAs before this manifest can be self-contained.

The same review compared the recorded source manifest with the current exact-8 hashes. Five files still match and three have drifted: `zircon_runtime/src/tests/runtime_absorption/input_stack/action_mapping.rs`, `docs/plans/zircon_runtime/runtime/12/2026-07-17-m5-input-event-bounds-current-source-closeout.md`, and this M4 record. Jobs `f6841642e70c4a43b8674c92f9f18461` and `c5d6303ce4334f3995b2b5073af7569b` remain truthful historical terminal evidence, but they are not acceptance evidence for the current frozen manifest. After both prerequisite SHAs enter HEAD, Runtime12 must freeze and re-attribute all eight files, rerun the source-bound action guard and canonical check, receive a fresh independent review, and only then request a managed milestone commit.

## Boundary

This follow-up corrects only Runtime12 M4 mirror authority and evidence wording. It does not alter the accepted bounded-retention/action-index implementation, promote the unrun `zircon_app` broad gate, change Runtime10 pointer-frequency ownership, or absorb Render01/Render05/Shader06/Performance01 paths.
