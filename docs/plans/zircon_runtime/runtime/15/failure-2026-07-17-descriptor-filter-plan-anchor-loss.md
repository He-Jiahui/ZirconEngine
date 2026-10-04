---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: descriptor-filter-plan-anchor-loss
origin_plan: docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md
fixing_plan: docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/02
fixing_child_dir: docs/plans/zircon_runtime/runtime/15
plan_link_mode: child_record_only
related_code:
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
  - docs/plans/zircon_runtime/runtime/index.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/engine-code-structure-convention.md
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/texture_descriptor_settings.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/runtime_dead_code/script_host.rs
tests:
  - cargo test -p zircon_runtime --lib --locked --jobs 1 --color never tests::runtime_absorption::structure_convention::production_file_budget::texture_descriptor_settings::runtime_15_texture_descriptor_settings_parser_is_child_owner -- --exact --test-threads=1
  - cargo test -p zircon_runtime --lib --locked --jobs 1 --color never tests::runtime_absorption::structure_convention::runtime_dead_code::script_host::runtime_15_script_host_value_descriptors_do_not_suppress_dead_code -- --exact --test-threads=1
  - cargo test -p zircon_runtime --lib --locked --jobs 1 --color never descriptor -- --test-threads=1
---

# Runtime15：descriptor filter plan/status anchors were lost

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md`
- 来源执行切片：M3 RuntimePlugin lifecycle `descriptor` focused gate
- 修复责任计划：`docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md`
- 交接原因：两项失败都验证 Runtime15 structure/status evidence routing；Frameworks02 不应删减结构守卫或复制 Runtime15 历史正文来通过自己的上行门。

## 失败现象与复现证据

Windows managed job `386e3d872b224189b488a0d37e564f34` / run `53f2e2c699974d9cb51bcbb9b3b040e5` 执行 `cargo test -p zircon_runtime --lib --locked --jobs 1 --color never descriptor -- --test-threads=1`，结果为 242 passed / 3 failed / 7996 filtered out，exit 101。本交接覆盖其中两项：

1. `runtime_15_texture_descriptor_settings_parser_is_child_owner`
   - 在第一个 `Runtime 15 plan` source 上短路，缺少完整 5-anchor 集合：切片标题、状态 slug、parent path、child settings path 与测试名。
   - 同一测试随后还会检查 runtime index、engine review/structure priority plans、module/importer/render-assets docs、status row、status map 与 date map；不能只补首个 panic 后就宣称完成。
2. `runtime_15_script_host_value_descriptors_do_not_suppress_dead_code`
   - 在第一个 `Runtime 15 plan` source 上短路，现有标题与测试名存在，但缺 `runtime_15_script_host_value_descriptors_coremin_check_passed`。
   - 同一测试还要求 runtime index、两个 priority plans、module convention 与 script host ledger 的三锚集合一致。

独立 current-source 静态矩阵确认：Runtime15 parent 对 texture 为 0/5、对 script 为 2/3；runtime index 对 texture 为 1/5、对 script 为 0/3；两个 priority plan 对 texture 均只有 parent path 1/5、对 script 均为 0/3。`docs/crates/zircon_runtime/structure/module-convention.md` 保留两组完整锚，script function ledger 也保留 script 3/3，说明生产/模块 owner 证据仍存在，断裂的是 current plan/status 路由。

## 最低共享层根因

Runtime15 parent/index 与 priority plan 的 current evidence aggregation 在压缩或 child-owner 拆分后没有同步迁移这两组可执行锚。测试仍以多个 aggregate documents 作为并列消费者，导致已有 child/module truth 无法通过 current parent/status route 被发现。

## 架构修复验收

- 先确定每组锚的 canonical current child record；parent/index/priority plans 应链接或由守卫读取该 child owner，不得重新复制大段历史成为第二事实源。
- 两个 exact tests 分别通过；两组完整状态 tuple 只由 `2026-07-17-descriptor-filter-plan-anchor-current-owner.md` 持有，守卫继续验证真实 production owner，并验证仍属公共契约的 module convention 与 script host ledger。
- 旧 Rust `plan_status` status row/status/date map 已由 2026-08-02 receipt-tree hard cut 物理删除，计划 lifecycle 由 Coordinator/Python tooling 持有；不得为满足本 failure 恢复这些退役 Rust 路径或未使用的文档读取。
- 重跑 Frameworks02 `descriptor` filter 时两项消失；Render07 的 SSR history 失败独立处理。
- 更新状态时保持 `engine-code-structure-convention.md` 与 `engine-code-review-findings-2026-06.md` 的优先级和 hard-cut 规则，不降低 required-anchor 集合。

## 禁止临时方案

- 不得删除测试、缩短 required anchors、从 `descriptor` filter 排除结构守卫，或用 ignored/allowlist 绕过。
- 不得把 archive aggregate 恢复为 current truth，不得在多个父计划复制同一完整状态正文，不得添加兼容路径。

## 修复结果与回传

- Current-source hard cut 已集成：`2026-07-17-descriptor-filter-plan-anchor-current-owner.md` 是两组 tuple 的唯一 current evidence owner；texture 与 script-host 两个守卫只读取该 child record，不再依赖 Runtime15 parent、runtime index 或两个 priority plans 的重复正文。
- 2026-08-14 前向复核确认 required-anchor tuples 分别为 texture 6/6、script-host 5/5；texture guard 的三个未使用文档读取已删除，script-host guard 保留 module convention 与 function ledger 的真实公共契约断言。旧 plan-status Rust tables 保持 hard-delete，父计划与优先计划未恢复兼容镜像。
- 当前仍为 `resolving_failure`：fresh immutable exact4 review、current-source managed Cargo `descriptor` gate 与 failure return 尚未完成，不声明 fixed。

## 2026-09-09 non-VM texture consumer continuation

The user has deferred `zr_vm`. This continuation covers the texture descriptor
consumer only; the script-host exact test and aggregate `descriptor` gate are
not submitted or claimed as accepted. Both original obligations remain in this
same lifecycle, without moving the VM half to a duplicate record.

The stable Session `failure-roll-01a07160-runtime15` acquired the archived guard
and failure record, plus the unchanged current-owner document, through transfer
fingerprint `2547f9e262b336b903028a228d63361090e98189d900ee20189fdbc3e2eb84ba`.
Snapshot 3295 preserves guard preimage
`a31c052460053ccbf7a77d0c3aeeb6b04303ffae0f054f023e851ddc975b90c8`
and document hash
`738ee159b99f4b4624d2960fed17eb57b043bba573be79776b90ec75e0279015`.
Input `runtime15-texture-descriptor-3295-20260909` contains 10,968 files,
manifest `218073b6480fe056d7d8d18c8c0a8153c7384512779f28c6db403c54f0f93aeb`.

Native managed job `e56418f45c2141958c37a623a737b2a9` ran the exact texture
test: 0 passed, 1 failed, 0 ignored, 6849 filtered out. The parent-source cohort
failed because `fn normalize_import_extent_fields(` has been replaced by the
current strict import-extent contract. The settings-child and final document
cohorts were not reached dynamically. Full artifact:
`results/runtime15-texture-descriptor-red-3295.{json,log}`.

The complete current producer chain was read before repair: descriptor
`apply_import_settings` rejects retired fields, parses non-zero depth/layers
and calls `apply_import_extent_settings`, which enforces dimension-specific
rules and cube face multiples. The old `ExtentSettingKeys` helper is gone.
The generic TOML/token parsing helpers remain in the settings child. Existing
descriptor tests cover the canonical single extent authority, zero values,
invalid dimension combinations, retired keys and render projection.

Snapshot 3298 changes only `texture_descriptor_settings.rs`, hash
`237ecd9606c7b199bd9d384d48ae3a164858128e5e7906bf92b282bc6227bf8a`.
It requires the actual canonical extent field, strict helper declarations and
both production calls. It rejects the retired normalization helper and
`ExtentSettingKeys` in both owners, retains every other parser-owner assertion
and both strict 800-line budgets, and leaves all six document anchors intact.
Parent and child code assertions now use the existing shared production view,
so comments, literals and test-only helpers cannot satisfy those code anchors.
The shared parser, production implementations, behavior tests and current-owner
document were not modified. Scoped rustfmt and whitespace checks pass.

Current main and the immutable input agree on the three direct producer/test
hashes: descriptor `cc6f7cf54e5a1c0401bbaa369030dd344cc3c0f51325cc2a3a0be4d7e56c1766`
(667 lines), settings `5adefcfa9f5603b649a14b2605cb065c86758d459f74b8237a926269d0cdd5e4`
(400 lines), and descriptor tests
`9082c9091dbf989e24ed2c28dddb5a58e81f08e5311db65d92c13c42a3791a31`.
The existing producer diffs were not acquired or attributed to Runtime15.
Final focused managed validation, fresh review and canonical return remain
pending; no serial harness or skipped VM acceptance is inferred.

The final input `runtime15-texture-descriptor-3298-20260909` is sealed with
10,968 files and manifest
`c78e80da5b1c6960c27dce48714dc3aa530c4dff210f3d027e39200202dfe579`.
Job `31beb7da597f443d9165bf58af3e1dcd` passed the exact structure guard:
1 passed, 0 failed, 0 ignored, 6849 filtered out. All parent, parser-child,
retired-owner, size and six document assertions executed. Artifact:
`results/runtime15-texture-descriptor-guard-3298.{json,log}`.

The preceding lower descriptor module job
`2b2e7158e28c4cbdaa15fff4c6517426` returned 26 passed, 1 failed,
0 ignored, 6823 filtered out. Every extent test passed; the metadata-token
fixture expected mip bias 0.5 despite not supplying that setting. This separate
Runtime04 test-input defect is recorded in
[its canonical fixed return](fixed-2026-09-09-texture-metadata-token-fixture-missing-mip-bias.md).
Artifact: `results/runtime15-texture-descriptor-lower-3298.{json,log}`.
Runtime04 snapshot 3302 adds the missing input field only. The final lower
batch, independent review and deferred VM/full-descriptor obligations remain
open; the passing structure guard is not whole-lifecycle acceptance.

## 2026-09-09 non-VM descriptor consumer closure

The deferred `zr_vm` instruction was applied to this lifecycle. Runtime04
received the texture metadata fixture as a separate lower-owner handoff and
sealed the resolver/migration/descriptor input chain. Its managed receipts are:

- migration resolver: job `fd6afaf6e96f4598830052d49c2ea5a5`, 4/4 passed;
- reference resolver: job `214ae209281d4e6986a4200b5214cbb7`, 3/3 passed;
- texture descriptor module: job `2c03b7c385ba4b5eb4e31dbec084ad19`, 27/27
  passed, including `import_settings_parse_texture_metadata_tokens`.

The final Runtime04 source input is
`E:/cargo-targets/zircon-engine/cache/build-benchmarks/runtime04-subasset-final-3305-20260909`
with manifest `3bfe12829402a3c0e72beaea5fdfff8d2d7c33125d97c8a8dc24f5b69899e15c`.
The existing Runtime15 structure guard job `31beb7da597f443d9165bf58af3e1dcd`
passed 1/1 against its unchanged guard snapshot; the fixture-only changes do
not alter that guard source. The first corrected fixture rerun is retained as
job `11f08f2ba0b34ffb90f5544136bb1009`, 26 passed and 1 failed, which exposed
the omitted `max_anisotropy = 8` before the final snapshot `3305`.

This closes the non-VM texture consumer evidence only. The script-host exact
test and aggregate `descriptor` gate remain deferred with `zr_vm`, and the
fresh zero-finding review, canonical `failure return`, closeout SHA and WeCom
notification are still required. The lifecycle remains open.

Runtime04 completed the fixture's canonical `failure return` in request
`6a8cd3a14dc649fe8edff6cd0f8cb06d`; the coordinator recorded delegated return
proof `9412d5b53e7b4127b91eb3a074a9beff` under the active Runtime15 destination
lease. The unique fixed artifact is linked above, and the original Runtime04
failure file was removed. This returns only the two missing fixture inputs;
the parent lifecycle's deferred VM/aggregate gates and independent review remain
open. No Git closeout or WeCom result is implied by the return.

## 2026-09-25 successor current-source static seal

Successor Session `failure-roll-01a084c8-runtime15-descriptor-filter-r1`
reconciled the six-path current boundary without editing production/runtime
owners. The canonical child record remains the sole tuple owner; the two
structure guards still read that record and retain their production/ledger
anchors. The inline source probe passed
`RUNTIME15_DESCRIPTOR_FILTER_CURRENT_SOURCE_PASS=3/3`. Exact guard
`rustfmt --edition 2021 --check` exited 0 and scoped `git diff --check`
exited 0 (only normal LF/CRLF conversion warnings were reported). Current
dirty changes in the plan, index, and two guard files were pre-existing
workspace overlays; their bytes were frozen rather than absorbed as a repair.

Authoritative pre-ticket snapshot `3865` records these six SHA-256 values:

```text
docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md 4aadfcecd06c9adccf21685e50131b767033256174a350f358b5579e156b1b1b
docs/plans/zircon_runtime/runtime/15/failure-2026-07-17-descriptor-filter-plan-anchor-loss.md e4a83061f8d74dbccdae77e211381b9592a26c41926b22e00d5e351d89ddfc6c
docs/plans/zircon_runtime/runtime/15/2026-07-17-descriptor-filter-plan-anchor-current-owner.md 738ee159b99f4b4624d2960fed17eb57b043bba573be79776b90ec75e0279015
docs/plans/zircon_runtime/runtime/index.md f0cc3fb48f96c07bf69fb1097a4242fa3f08e879045b2cdd5ab29ae4d822efd1
zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/texture_descriptor_settings.rs 237ecd9606c7b199bd9d384d48ae3a164858128e5e7906bf92b282bc6227bf8a
zircon_runtime/src/tests/runtime_absorption/structure_convention/runtime_dead_code/script_host.rs 3906cd6e772c2630e8180b5f5682399b6fed758ac7d210f64807ae5591039a50
```

## 2026-09-25 static validation ticket receipt

Coordinator ticket `9854ef4f4029494597358f8e8b3dbfcd` (submit request
`81017f3363584f94b97d63bc1947b6c5`, caller request
`runtime15-descriptor-filter-static-20260925-r1`) is queued with
`executionKind=pending`. Its source manifest is the six-path snapshot `3865`;
the command is the exact inline Python probe recorded in the coordinator
receipt, with `cargo` and `rust` disabled, `staticParseOnly=true`, and
`fullCoverage=false`. Admission correctly retained the Runtime09/Render01/
Render07/Plugins13 dependency blockers. No Cargo test, exact filter, aggregate
`descriptor` gate, VM acceptance, or upward product gate was executed or
promoted by this ticket.

## 2026-09-25 independent static review

Reviewer `/root/review_editor03_gizmo_private` reviewed authoritative snapshot
`3865` and found `Critical=0`, `Important=0`, `Moderate=0`. The review
confirmed the child record is the only current tuple owner (parent/index do not
duplicate the historical body), the `3/3` marker and formatting evidence match
the six-path manifest, and ticket `9854ef4f4029494597358f8e8b3dbfcd` is only
queued/pending. It also confirmed that exact texture/script/aggregate Cargo,
`zr_vm`, upward product, fixed return, and closeout gates remain open.
