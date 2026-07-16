---
handoff_kind: fixed
status: fixed
created_at: 2026-07-16
summary_slug: plan-output-archive-test-anchor-drift
origin_plan: docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
fixing_plan: docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/05
fixing_child_dir: docs/plans/zircon_runtime/runtime/15
origin_workflow_node: M3
priority: 100
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/tests/runtime_absorption
  - zircon_runtime/src/tests/runtime_absorption/dynamic_scene/sources.rs
  - docs/plans/_archive/zircon_runtime/runtime/05/2026-07-09-scene-editor-boundary-closeout-output-records.md
  - docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md
  - docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md
  - docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md
  - docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md
tests:
  - cargo test -p zircon_runtime --lib --locked text::sdf::font_bake::tests::sdf_cpu_preparation_caches_shaped_metrics_across_frames -- --exact --nocapture
resolved_at: 2026-07-16
---


# Runtime 15：计划输出归档后的测试锚点漂移

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md`
- 来源执行切片：M3 Text SDF CPU preparation cache 精确回归门
- 修复责任计划：`docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md`
- 交接原因：最低共享根因是 Runtime15 将四个 2026-07-09 输出记录硬切到规范 `_archive` owner 时，没有同步其自身 `runtime_absorption` lib-test 的编译期文档锚点；Frameworks05 的 SDF 实现和断言尚未进入编译/运行阶段。

## 失败现象与复现证据

- Windows 受管 Cargo job：`9132729e7f6a4263a3c24632d6b31c85`。
- 日志：`E:/ZirconBuilds/frameworks05-m3-sdf-face-resolution-cache-20260716.log`。
- 结果：job 于 2026-07-16 01:49:38 +08:00 自然释放，exit code `101`；`zircon_runtime` lib-test 因 `268` 个编译错误终止，Frameworks05 精确测试未运行。
- 编译器一致报告 `include_str!` 无法读取 Runtime15 旧 child path 下的四个 2026-07-09 输出记录；这些文件当前已删除，内容存在于 `docs/plans/_archive/zircon_runtime/runtime/15/`。
- Runtime15 当前 child directory 没有覆盖该漂移的 open failure；旧输出文件删除与归档副本属于 `runtime15-priority-output-archive-20260716` 切片。

### 2026-07-16 current-source follow-up

- Frameworks05 managed job `569c45965b044e79992abd286dd3f08c` 以 exit `101` release，未进入任何 Text 断言。该轮编译共报告四项：本 Failure 的 Runtime05 输出记录缺失、Render18 的 volumetric helper 未导出，以及两个 Text prepare-report fixture 缺少 `font_faces_changed`。
- 两个 Text fixture 已在最低 owner 修复；Render18 随后从 froxel 父模块导出 helper。fresh managed job `70627811c1204085a79ca1ef08772262` 已通过 default production 与 target-server 双编译，分别耗时 4m17s 与 1m19s，证明生产图不再受上述三项阻断。
- `zircon_runtime/src/tests/runtime_absorption/dynamic_scene/sources.rs` 当前已把 Runtime05 scene-editor closeout 与 Runtime15 runtime-index 两个遗留 anchor 直接硬切到规范 `_archive` owner；Runtime absorption 范围内五个旧 active Rust anchor 的静态扫描为 `0`。Failure 仍保持 open，直到原始 default lib-test 精确复现实际编译并执行 Text SDF 断言后再回传 fixed。

## 最低共享层根因

Runtime05/Runtime15 的五个输出记录 owner 已从活动 child directory 硬切到 `_archive`，但 `zircon_runtime/src/tests/runtime_absorption/**` 中的编译期 `include_str!` 没有在同一迁移中原子收敛。归档 owner 与测试 source anchor 一度指向不同事实源，导致任意 `zircon_runtime --lib` 测试编译被全局阻断。

## 架构修复验收

- 将 Runtime05 scene-editor closeout 与四个 Runtime15 输出记录的测试锚点全部硬切到规范归档 owner，或按 Runtime15 当前测试架构收敛为单一受管 locator；五个旧 child path 引用必须归零。
- 不恢复五个已归档文件到活动 child directory；归档位置仍是唯一事实源。
- 运行 Runtime15 聚焦 source-anchor/static guard，证明五个旧路径引用为 `0`，五个规范归档文件均可解析。
- 重新运行上述原始受管 Cargo 精确复现，要求成功编译并实际执行 Frameworks05 测试。
- 原始复现通过后通知 Frameworks05 M3 继续 Text SDF 回归、产品帧和性能门。

## 禁止临时方案

- 不得恢复五个旧文件、增加别名、兼容 re-export、shim、silent fallback、重复事实源或测试专用绕过。
- 不得弱化或删除 Runtime15 文档锚点测试来隐藏迁移不完整。
- 不得在 Frameworks05/Text 生产代码中增加与该文档路径漂移有关的调用点特例。

## 修复结果与回传

- 根因：Runtime05/Runtime15 five plan outputs were hard-cut to canonical _archive owners without atomically updating every runtime_absorption include_str anchor.
- 架构修复：Hard-cut the remaining Runtime05 scene-editor and Runtime15 runtime-index compile-time anchors to the canonical archive paths; all five old active-path anchors are now zero, with no alias, shim, restored duplicate, or test bypass.
- 验证：Static audit: 0 old active anchors and 5/5 archive files. Independent review: Critical 0, Important 0, Minor 0. Managed job eaebe4e27b7c4f6ab267a512c0854a2b ran the original exact default lib-test and released exit 0: 1 passed, 0 failed, 8168 filtered; test body 4.07s.
- 回传：Runtime15 archive-anchor drift is fixed and returned; Frameworks05 M3 may resume its Text default/UI and graphics-only upward gates.
