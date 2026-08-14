---
handoff_kind: failure
status: open
created_at: 2026-08-01
summary_slug: assert-contains-all-archive-fallback-masking
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/15
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/support.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention
  - tools/tests/test_runtime_receipt_hard_cut.py
tests:
  - assert_contains_all rejects anchors absent from the supplied source
  - Runtime15 historical guards name and read one explicit canonical owner
---

# Runtime15 assert_contains_all archive fallback masking

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：Runtime15 structure-convention assertion helper audit
- 修复责任计划：`docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md`
- 交接原因：结构守卫 assertion helper 与其调用点属于 Runtime15 测试基础设施，Performance01 只记录其可导致 false green 的审阅结论。

## 失败现象与复现证据

`support.rs::assert_contains_all` 根据自由文本 `label` 选择额外 archive/current-owner inventory，并在调用方传入的 `source` 缺少锚点时从额外内容补齐。Sol/High 静态复刻确认 1,392 个 literal 调用会触发 fallback，覆盖 692 个文件、720 个函数；另有 705 个函数通过动态 `label` 调用，其中 472 个函数仍同时读取四份退役 live root，41 个函数虽已读取四份 canonical archive 却仍使用非精确 helper。八文件迁移后的只读扫描显示 Runtime15 parent、runtime index、engine review 与 engine structure 的 live-root 读取仍分别散布在 504、484、621、616 个结构守卫文件中，对应 536、510、661、655 次读取，因此错误 source 可以被 helper 静默掩盖。`runtime_15_output_archive_source` 实际还会拼接 active `runtime/15/*.md`，包括 open failure，而不是只读取 `_archive`；新增 failure 本身即可改变 fallback 结果。

## 最低共享层根因

assertion helper 同时承担“检查已传入 source”和“猜测另一个证据 owner”两种职责，source identity 没有进入类型或调用合同；`runtime_15_output_archive_source` 的名称还与其实际读取的 active child directory 不一致。

## 架构修复验收

- `assert_contains_all` 只检查调用方显式传入的 source，或者由 typed owner API 返回带固定身份的 source；不得按 label 猜 owner。
- 分批把历史守卫硬切到明确 canonical archive，把 current-state 守卫保留在 live/index/current-owner 文件，并为每批保留 failure/fixed 与真实执行证据。
- 新增 helper regression：传入错误 source 时必须失败，即使其他 archive 含有同名锚点。
- 删除仅由 label fallback 使用的 `current_status_row_owner_inventory_source`、`runtime_15_output_archive_source`、`engine_code_review_findings_archive_source` 与 `engine_code_structure_archive_source`；保留仍被调用方显式使用的 `priority_plan_doc_current_owner_archive_source`，不得把 typed owner 聚合与隐式 fallback 一并误删。

## 禁止临时方案

不得一次性删除 fallback 后以大面积 ignored/allow-failure 收场，不得把 archive 内容复制回 live 文档，也不得把 label 匹配扩成更多隐式 owner。

## 修复结果与回传

Open state: `八个守卫文件的 historical cohort 已硬切 canonical archive + exact helper；41 个已读 canonical archive 的函数和 67 个非 status literal 误触发点是下一优先批，其余调用点仍须按 historical/current owner 分批归因后删除 label-based fallback。未执行 Runtime15 Cargo 目标测试，当前不得宣称 helper false-green 风险已关闭`。

2026-08-01 的第二批 Sol/High 精确归因把五个文档镜像 cohort 改为 `assert_contains_all_exact`：`plugin_bridge_table_reports.rs:124`、`render_frame_extract_geometry.rs:79`、`render_scene_world.rs:87`、`render_post_process_screen_space_reflection_tests.rs:89` 和 `native_plugin_loader.rs:97`。只读复核确认这些 cohort 的显式 source 共覆盖 158/158 个要求锚点；生产源码、状态表和 current-owner 断言未改。固定 Rust 1.94.1 rustfmt、scoped `git diff --check` 与精确调用计数均通过，但尚无 Runtime15 Cargo GREEN，因此 failure 继续保持 `open`。

同批审阅还确认两处真实 false green：`structure_convention/lock_poison_policy/runtime_services/plugin_bridge.rs:58` 与 `structure_convention/script_vm_lock_poison.rs:65` 显式传入的 `m3/lock_poison_status.rs` 均缺少断言要求的全部五个锚点，当前通过 label fallback 补齐。Sol/High 精确复核后，VM guard 已改读 canonical child `m3/lock_poison_status/script_vm_recovery.rs`，生产源码、挂载源、四份 archive、两份 current doc 和状态 child 共覆盖 44/44 个显式锚点，三个 assertion 均已切换 `assert_contains_all_exact`。

Plugin guard 不能做同样的机械迁移：当前 `plugin/bridge/table.rs` 已由 `Mutex` provider 槽硬切到 `ArcSwap<BridgeEntryState>`，旧 poison guard 的九个生产锚点为 0/9；live 文档与 `runtime_services_recovery.rs` 状态 row 仍描述已淘汰的 mutex recovery 契约。该语义由活跃 `plugins01-plugin-lock-recovery-r5-20260801` 及既有 bridge failure 负责，已通知其退役旧 Runtime15 guard、current status row 与挂载/meta-guard 引用，或按 ArcSwap stable-snapshot 契约重建守卫。四份 archive 仍是历史事实，不应重写。Runtime15 helper failure 继续保持 `open`，且尚无 Runtime15 Cargo GREEN。

VM guard 的受管精确验证 wrapper job `c12a661fdc1245f6b314750802b51c50` 在本地 10 分钟等待上限后失去外层结果；Cargo 子进程随后自然退出，但 coordinator 没有生成可读取的 run 结果。该 job 已按真实 wrapper exit `124` 收尾并释放，不能作为 pass/fail 证据；当前只保留 44/44 静态锚点、3 个 exact 调用、0 个 fallback 调用的验收。

第三批 Sol/High 精确归因修复了 `structure_convention/native_live_host_lock_poison.rs`：守卫不再读取已经迁空的 live Runtime15 plan/index/engine root，也不再读取只含 7/35 锚点的 `m3/lock_poison_status.rs` 父文件；它现在显式读取四份 canonical archive 与 typed `m3/lock_poison_status/runtime_services_recovery.rs` child。三个 assertion 全部切换为 `assert_contains_all_exact`，七个 owner source 的静态锚点为 35/35，fallback 调用为 0；rustfmt 与 scoped diff-check 通过。

对应的受管 Windows `validate-matrix` 运行使用 `--locked`，但在执行目标 `runtime_15_native_live_host_bridge_methods_lock_poison_recovery_guard_covers_binding_registry` 前被当前脏 lib-test 源的 49 个外部编译错误阻断，首个错误来自活跃 Runtime04 scene session 的 `artifact.rs` 重复 `io` import，另含 shader prewarm export、text atlas/cache、render-framework visibility 与 UI-surface import 漂移。目标文件没有 diagnostic，且本轮未修改这些外部 owner；因此该批只形成静态修复证据，failure 继续保持 `open`。

第四批 Sol/High 审阅清除了另一类同根 false green：`backend_owners.rs` 与 `provider_boilerplate/full_audit.rs` 不再把 `runtime_15_plan.clone()` 重命名成虚构的 `render_index` 后作为第二份证据；`f12_current_state.rs` 与 `gate_wording.rs` 不再把同一 code-structure archive clone 成 `runtime_index` / `structure_convention`，而是分别读取真实的 `2026-07-09-runtime-index-output-records.md` 与 `2026-07-09-engine-code-structure-output-records.md`。Runtime15 structure guards 的文档 owner clone-alias 扫描现为 0，四文件 rustfmt 与 scoped diff-check 通过；本批未执行 Runtime15 Cargo 目标，不能把静态归因写成 GREEN。

同批发现 canonical `2026-07-09-code-structure-and-module-conventions-output-records.md` 仍记录 OffscreenTarget 保留 `9` 个 texture owner，而当前源码常量与精确测试均为 `10`（`final_color`、`global_illumination`、`scene_color`、`bloom`、`gbuffer_albedo`、`gbuffer_emissive`、`gbuffer_material`、`normal`、`ambient_occlusion`、`depth`）。历史 archive 不在本 failure 中重写；Runtime15 owner 必须在 current owner/fixed return 中明确解释或修正该 9/10 漂移后，才可关闭本记录。

## 2026-08-03 hard-cut 进展

源码实现已删除 label 驱动的隐式 fallback 与仅供其使用的 status owner helpers；`assert_contains_all` 现在只检查调用方显式传入的 source。更高优先级 receipt hard cut 同时删除了纯历史状态镜像守卫，而生产结构守卫继续保留。当前本地 hard-cut/Runtime03 Python 回归 5/5 通过，独立二次审查为 Critical/Important/Minor = `0/0/0`；managed Runtime lib-test/plan-output 证据仍待回执，因此本 failure 保持 `resolving`，不提前改为 fixed。

## 2026-08-14 current-source 量化复核

- `structure_convention/support.rs::assert_contains_all` 无条件委托 `assert_contains_all_exact(label, source, required)`；调用方传入错误 source 时不再可能由 label 或其他 archive 补齐。当前 642 个 structure guard Rust 文件中共有 2,096 个 `assert_contains_all(` 文本 occurrences，其中 2 个是 helper definitions，真实 call expressions 为 2,094；它们全部采用显式 source 语义，旧记录中的 1,392 fallback-trigger 计数不再适用。
- 四个仅供隐式 fallback 使用的 owner helpers 保持物理删除。`priority_plan_doc_current_owner_archive_source` 共 7 个文本 occurrences：root owner 与 local wrapper 共 2 个 definitions，另有 5 个调用引用（1 个 assembly、1 个 delegate、3 个 guard consumers）；它们均为显式 priority inventory 路由，不参与 `assert_contains_all` 的 source 选择。
- `tools/tests/test_runtime_receipt_hard_cut.py` 新增防回归：对 wrapper 和 exact helper 两个完整函数体做 whitespace-normalized exact equality，确保 wrapper 只能委托 exact helper、missing 只能从传入 `source` 计算；同时禁止四个退役 implicit owner helpers 回归，并明确保留显式 priority inventory helper。
- 当前仍为 `resolving_failure`：fresh Python execution、immutable review、managed Runtime focused evidence 与 failure return 尚未完成，不声明 fixed/accepted。

## 2026-08-15 focused static regression

- `PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_runtime_receipt_hard_cut -v` executed `4` tests with `4 passed; 0 failed`. This includes the exact source-contract regression for the wrapper and the retired implicit-owner helpers.
- A Rust 1.94.1 standalone harness on `D:\\ZirconBuilds` exercised the real helper with a label containing the required anchor but a supplied source without it. The helper panicked as required and the harness completed `1 passed; 0 failed`; an unrelated label with a supplied anchor passed.
- `git diff --check` for the test and this record has no whitespace defect. The dynamic Runtime focused gate, independent immutable review, and canonical failure return are still outstanding, so this handoff remains `open`.
