---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: registration-filter-plan-anchor-loss
origin_plan: docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md
fixing_plan: docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/02
fixing_child_dir: docs/plans/zircon_runtime/runtime/15
plan_link_mode: child_record_only
related_code:
  - docs/plans/zircon_runtime/runtime/15/2026-07-17-registration-filter-plan-anchor-current-owner.md
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/provider_boilerplate/registration.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/core_runtime_registration.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/support.rs
  - docs/crates/zircon_runtime/structure/module-convention.md
  - docs/crates/zircon_runtime/core/runtime/lifecycle.md
  - docs/crates/zircon_runtime/graphics/runtime_provider/registration.md
tests:
  - cargo test -p zircon_runtime --lib --locked --jobs 1 --color never tests::runtime_absorption::structure_convention::provider_boilerplate::registration::runtime_15_provider_registration_uses_shared_owner -- --exact --test-threads=1
  - cargo test -p zircon_runtime --lib --locked --jobs 1 --color never tests::runtime_absorption::structure_convention::test_file_budget::core_runtime_registration::runtime_15_core_runtime_registration_structure_tests_are_folder_backed -- --exact --test-threads=1
  - cargo test -p zircon_runtime --lib --locked --jobs 1 --color never registration -- --test-threads=1
---

# Runtime15：registration filter plan/status anchors were lost

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md`
- 来源执行切片：M3 RuntimePlugin lifecycle `registration` focused gate
- 修复责任计划：`docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md`
- 交接原因：两项失败都验证 Runtime15 structure/status evidence routing；Frameworks02 不应删减守卫、恢复 archive aggregate 或复制 Runtime15 历史正文来通过上行门。

## 失败现象与复现证据

Frameworks02 Windows managed job `cbbe13aff0db495181c4ec16e984c51f` / run `a4e893e4be0c44859e38fc19b0697986` 执行 `registration` filter，结果 211 passed / 5 failed / 8025 filtered，exit 101。其中两项 Runtime15 structure guards 都在第一个 `Runtime 15 plan` aggregate source 上短路：provider-registration 缺完整 3-anchor tuple，core-runtime-registration 缺完整 6-anchor tuple。

## 最低共享层根因与硬切

两项生产/模块/status truth 均仍存在；断裂的是 current plan aggregation。修复必须新增单一 current child record，并让两个守卫 exact 读取该 owner；不得把完整 tuple 复制回 parent/index/priority plans，也不得继续由 `assert_contains_all` 的 archive fallback 隐式满足。

## 架构修复验收

- 两个 exact tests 通过，并真实检查唯一 current child tuple 与仍存活的 module/provider/lifecycle 公共契约。
- 旧 Rust `plan_status` status/date rows 已由 2026-08-02 receipt-tree hard cut 物理删除，计划 lifecycle 由 Coordinator/Python tooling 持有；不得为本 failure 恢复退役状态路径。
- Frameworks02 `registration` 重跑时这两项消失。

## 禁止临时方案

- 禁止删减 required anchors、排除 filter、恢复 archive aggregate、兼容 alias/shim 或重复父计划正文。

## 修复结果与回传

Open state: `focused_guards_passed_review_and_upward_acceptance_pending`.

- 2026-08-14 current-source 复核确认 `2026-07-17-registration-filter-plan-anchor-current-owner.md` 是 provider-registration 与 core-runtime-registration 两组完整 tuple 的唯一 current evidence owner；父计划、runtime index 与 priority plans 没有恢复重复正文。
- 两个 structure guard exact 读取该 child record，并继续验证真实 provider/module/lifecycle owner；core guard 对 module convention 验证切片、状态与 folder `mod.rs` 三锚，对 lifecycle doc 验证同组三个锚、两个 focused child owner 与 exact guard 名共 6 锚。共享 `assert_contains_all` 只检查显式 source，没有 label-based archive fallback。required child tuples 分别保持 provider 6/6、core-runtime 7/7。
- current child record、两条 guard 与共享 helper 相对 HEAD 零差异；fresh immutable exact review、managed current-source `registration` gate 与 failure return 尚未完成，因此不声明 fixed/accepted。

### 2026-09-08 validation input closure continuation

Fixing Session `failure-roll-01a07160-runtime15` continues this lifecycle;
historical ticket ownership remains unchanged. Transfer fingerprint
`bfc99d7bfc23da29044eaf660f165eaefd0868d114530d4f4374f0ade202330f`
acquired the cancelled/archived-owner scope after all eight paths were verified
unchanged against HEAD `695e4e58987a1ac82111ac700f4e23b1ba52d74e`.
Preimage record snapshot 3258 preserves the historical evidence. No guard,
production implementation or documentation content was changed.

The previous broad registration job's two failed guard names did not establish
a recurrence of missing anchors: its sealed input lacked the current child
record, lifecycle document and provider-registration document; its module
convention document was stale. A focused managed Windows static/no-default/locked
diagnostic, job `1512a5a90d964b59afa52ae787a3d8cf`, executed the provider guard
on input `runtime04-junction-3251-20260908`, manifest
`55328b53070a78a8374cf55f629fc8501d01dee2f2517a2ddc2db94fbf1bfca9`:
0 passed, 1 failed, 0 ignored, 6849 filtered out. It failed while opening the
missing `2026-07-17-registration-filter-plan-anchor-current-owner.md`, with
Windows OS error 3, before any anchor assertion. The complete receipt and
diagnostic are `results/runtime15-registration-doc-input-diagnostic-3251.{json,log}`.

Snapshot 3257 freezes the three unchanged Rust guard/helper files and all four
runtime-read documents. The three Rust files and 14 directly inspected
provider/registration source files match the parent input exactly. The
derived input therefore repairs only the document closure. The three module
documents are now explicitly listed in this record's `related_code` so future
source-bound validation includes these actual test inputs. Required tuples
remain provider 6/6 and core-runtime 7/7; the retired Rust status tables and
implicit archive fallback remain absent. Both target tests still require
successful execution on the complete input, independent review and formal
acceptance. The full registration batch also retains the separate
[Runtime11 observer hang](../11/failure-2026-08-23-task-terminal-delivery-bounded-dispatch.md).

### 2026-09-09 complete-input focused results

Input `runtime15-registration-input-3257-20260908`, manifest
`86719cb85fc2f0b13f168beea45a0271dade59f04cfbb1aea23ac279d2c913c2`,
contains 10,965 files. Its provenance binds snapshot 3257, adding the three
missing documents and refreshing the stale module-convention document; no
Rust bytes differ from the diagnostic input. Both fully qualified filters
executed their intended target, with 1 passed, 0 failed, 0 ignored and 6849
filtered out in each managed Windows static/no-default/locked run:

- Provider guard: job `9703666222fa47e98322ebb455cfa814`,
  `results/runtime15-provider-registration-3257.{json,log}`.
- Core-registration guard: job `6a5dfc60b7d945d5972db7337f65a456`,
  `results/runtime15-core-registration-3257.{json,log}`.

The source-manifest verification passed after each run. These results confirm
all original focused structure and anchor assertions on the complete input;
they do not claim the separate broad registration batch passed. The historical
root cause is already repaired in HEAD, and this continuation supplies missing
input metadata and dynamic evidence. Fresh independent review, formal
fixing-Session binding, canonical return and closeout remain pending.

The existing independent task `优化协调器验证效率` completed review of
snapshot 3257 and final record snapshot 3262: source Critical 0, Important 0,
Moderate 0; evidence Critical 0, Important 0, Moderate 0. Report:
`.codex/tmp/runtime15-registration-3257-review-20260909-result.txt`.
All eight current, attributed and ObjectStore hashes matched before and after
review, with no reviewed-path ownership conflict. The module-convention
document's CRLF checkout also matched HEAD after Git line-ending normalization.
This supersedes the fresh independent-review requirement above for these
exact bytes. The known archived-reviewer Session gate still prevents formal
review binding; full registration, canonical return and closeout remain open.
## 2026-09-25 successor current-source static seal
The stable Session `failure-roll-01a084c8-runtime15-descriptor-filter-r1`
continued this registration-filter lifecycle after the descriptor and
dynamic-API handoffs. Eight archived-owner paths were transferred with audited
fingerprint `3ea00307103a0b4d97cc3cb8f5fba7798f53c10e7819cc73cd9cf6ff1dfede91`;
no production implementation was edited. The current-source probe passed
`RUNTIME15_REGISTRATION_OWNER_CURRENT_SOURCE_PASS=2/2`, covering both canonical
child tuples and both exact structure guards. `rustfmt --edition 2021 --check`
exited 0 and scoped `git diff --check` exited 0 (normal LF/CRLF warning only).
Pre-ticket snapshot `3873` froze the nine-path manifest and hashes. The
coordinator snapshot is authoritative for the complete values and path order;
all nine current bytes matched before ticket submission.
## 2026-09-25 static validation ticket receipt
Coordinator ticket `74213ff2ca3e43039eb5fa7171bb12e0` (request
`runtime15-registration-owner-static-20260925-r1`) is queued with
`executionKind=pending`. Its nine-path manifest is snapshot `3873`; the exact
Python owner probe is recorded in the ticket with Cargo and Rust disabled,
`staticParseOnly=true`, and `fullCoverage=false`. Admission retained the
Runtime09/Render01/Render07/Plugins13 dependency blockers. The historical
managed focused jobs cited above remain evidence for their old immutable
inputs; this ticket is a current-source static receipt only. Full
`registration` Cargo, upward Frameworks02, fixed return, closeout, and WeCom
remain pending.

## 2026-09-25 independent static review

Reviewer `/root/review_editor03_gizmo_private` rechecked the corrected
registration-filter record against source boundary `3873` and wording-only
snapshot `3875`: no literal `\\n+` patch markers remain; the 2/2 marker,
formatting evidence, nine-path manifest, queued ticket
`74213ff2ca3e43039eb5fa7171bb12e0`, and pending dynamic/upward/closeout gates
are coherent. Findings are `Critical=0`, `Important=0`, `Moderate=0`.
