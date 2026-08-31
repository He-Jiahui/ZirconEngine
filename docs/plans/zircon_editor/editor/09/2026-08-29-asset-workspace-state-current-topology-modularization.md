---
status: current-topology-review-complete-fresh-ownership-cross-module-parent-blocked
created_at: 2026-08-29
implementation_status: test-owner-extracted-static-guard-fixed
managed_validation_status: not-submitted-current-parent-and-mixed-api-closure-incomplete
source_baseline_sha256: 2B243ECD4233C602C0917B107F9972CF2ECA560E85FAF2D35B134357828D7A7D
related_code:
  - zircon_editor/src/ui/workbench/project/asset_workspace_state.rs
  - zircon_editor/src/ui/workbench/project/asset_workspace_state/performance_tests.rs
  - zircon_editor/src/ui/workbench/snapshot/asset/asset_workspace_item_generation.rs
  - zircon_editor/src/ui/workbench/snapshot/asset/asset_workspace_snapshot.rs
  - zircon_editor/src/ui/workbench/snapshot/asset/mod.rs
  - zircon_editor/src/ui/workbench/snapshot/mod.rs
  - zircon_editor/src/ui/workbench/shell_state.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/asset_access.rs
  - zircon_editor/src/ui/layouts/views/asset_browser/logical_paint_source.rs
  - zircon_editor/src/ui/retained_host/asset_pointer/content/layout.rs
  - zircon_editor/src/tests/editing/asset_workspace.rs
---

# Editor09 asset workspace state current-topology modularization

## 结论

本轮只基于共享 current source 工作。实施前 `asset_workspace_state.rs` 的 SHA-256 为 `2B243ECD4233C602C0917B107F9972CF2ECA560E85FAF2D35B134357828D7A7D`，共 830 行；历史 `editor09-asset-workspace-projection-20260814` 已归档，其旧 blob 未被读取、复用或覆盖到当前文件。

current root 同时承载 production state/projection 与 5 个内联性能回归，已经超过 800 行 owner review 预算。本轮将测试原样迁移至标准 Rust 子模块 `asset_workspace_state/performance_tests.rs`，根文件只保留 `#[cfg(test)] mod performance_tests;`。迁移后根文件 721 行，测试 owner 114 行；新增第 6 个结构回归，要求 production root 保持不超过 800 行且禁止重新内联测试模块。

这不是性能优化：production API、state transition、projection/cache 算法和可见资产语义均未改动，也没有生成 CPU、alloc、GPU 或功耗收益声明。现存 asset pane/catalog/watcher/import/dirty-registry 等性能 failure 继续遵循“先 whole-module review、profile baseline、hotspot report，再实施算法变更”的门禁。

## Current integration boundary recheck（2026-08-31）

新 Session `root-editor09-asset-workspace-current-topology-20260831` 已按当前三文件重新登记 ownership；transfer request 为 `2e4410629b824608a1d83c269b1295ef`，没有使用 maintenance override，也没有复用归档 Session 的旧 blob。当前精确哈希仍为 root `63F97EE4BA3F299FAC7796D2A99FA665AD2234DF3FF044651A53E0E1466071FA`、test owner `6FDF7D380A283ECC433E833C33CE611FC121B34600EB874BA023EF7628913926` 与本记录 `23D721EA910664FFEC56B3DA8831AEB753AFAB66E7211B87FD914F25B007200D`（本节写入前）。

该三文件 scope 不是可独立提交的 Rust closure。current root 已消费未跟踪的 `snapshot/asset/asset_workspace_item_generation.rs`，并要求 current `asset_workspace_snapshot.rs`、`snapshot/asset/mod.rs` 与 `snapshot/mod.rs` 发布同一 ABI；新 generation 又被 asset access、Browser logical paint/selection、retained pointer/drag、editing tests 与 shell cache 等 12 个 current-source consumer 使用。其中 `ui/workbench/shell_state.rs` 已位于活跃 Editor08 Session 的 write scope，不能转移或吸收到本 Session。

因此本 Session 只声明 current-owner 重建、测试 owner 拆分与静态结构门，不创建缺少 ABI consumer 的 isolated integration candidate。后续必须在 Editor08 释放 `shell_state.rs` 后重新冻结完整 current closure，或等各 owner 先集成到 HEAD，再提交本 slice。当前没有可运行的 current Editor binary 或 RenderDoc capture，动态 1/1k/10k、CPU/alloc/GPU/功耗数据仍不可取得；不得据此继续修改缓存算法或声称性能收益。

独立复审还确认迁移后的既有 source guard 会确定性失败：`asset_snapshot_normalizes_search_once_and_streams_parent_paths` 扫描整个 production root，却把完整 projection 与 catalog 增量 patch 两条独立执行路径中的 lowercase 调用合并计数。将 5 个测试逆向内联回实施前文件后可精确还原 baseline SHA，证明该失配不是本轮模块拆分引入。2026-08-31 后续修复没有把期望值机械放宽为 2，也没有在缺少 profile 的情况下新增 normalized-query cache；测试改为分别截取 `build_snapshot` 与 `patch_catalog_item_generation` 方法体，要求每条路径各规范化 1 次，从而验证真实执行边界。后续优化仍必须先比较“查询状态预编译 / projection key 持有 / rebuild 与 patch 各自规范化”三种整体边界，并以 1/1k/10k 资产的 CPU、alloc 与交互延迟数据决定语义门。

## Fresh current ownership 与完整模块重审（2026-08-31）

前述三路径 Session `root-editor09-asset-workspace-current-topology-20260831` 的 write scope 不可变，且只能证明局部类型引用，不能证明 Cargo materialization 闭包。它已通过 request `1c43351f50aa49aeb287c339074d31b5` 标记为 `cancelled`，没有回滚、覆盖或删除共享工作树内容。随后建立 successor `root-editor09-asset-workspace-current-closure-20260831`；注册 request `2c8791f7b44f4e79b4e9df07d70f1ab6`，fresh transfer preview request `426475eec7214af68dd5b88a8bbc95f2`，fingerprint `e2ab8bc6cb5bb3f7634bc188f0fd99378fd279dc8daf2061d06831c6cfa280f7`，apply request `ca602ba8267a41e0914a8e7f628c9b11`。12 个 current blob 均为 `eligible`，未使用 maintenance override，也未复用归档 Editor09 的旧 preview。

fresh transfer 后的精确 SHA-256 为：

- editing test `3392901F5A4FB494712CBC8774E33BBCB439E663EFC4E08EDBD42E36868333E5`
- host asset access `A0C569AB75ADBE98ED0E7F6F5C8B7588BA2FAE29BAFE61556732636C6D1CDF47`
- logical paint source `F28698C31CA6C71BFF61ED617B040C61BDBDEBF403F6B6CEAE83C0A21E89792B`
- retained pointer layout `FA0763BACE34D260E381BFAE10572709E99807F83BDEB65C0F365F33AAF273F2`
- workspace state `63F97EE4BA3F299FAC7796D2A99FA665AD2234DF3FF044651A53E0E1466071FA`
- performance test owner `AB0FC244CE5A3A114B97A2B94F24CA02B0885CC975FD1A175DFD112D9E56B11E`
- shell cache `41330D15C563A6E27098BD3B7FC0060B0248A308658A09145362D276EA42DFC3`
- item generation `720EB1BCD5763C5CFF0D033D909D87496685D8ADC12917A6A1E82BC43C4D3A36`
- workspace snapshot `C63FEB4DC57F3E03EB2C5579038CCD389FF4524DA9316CB0F9D2A0A239C0D80C`
- asset re-export `13BF439B3F91DB90DB636A3CC038DAEEA657D2CF53786C96F21A20E1FD81E160`
- snapshot re-export `5667BB03BADB1F20A576C5A9AD1538D589CBE0427A15335609B3CD4C83EA196A`

进一步的 module-declaration 与 API call-graph 审计推翻了“12 路径已经是完整闭包”的假设。`logical_paint_source.rs` 当前仍是 untracked 文件，只有 scope 外、相对 HEAD 已修改的 `ui/layouts/views/asset_browser.rs` 才声明 `mod logical_paint_source;`；该 parent 又与 20 个 tracked asset-browser 子文件、拆分后的 tests 目录、virtualization、render-source-frame 和 UI performance counter 同时变化。将 logical paint 文件单独覆盖到 validation copy 只会产生一个未被编译的孤立文件，不能形成有效证据。与此同时，current `asset_access.rs` 还混入 typed error/public return contract 改造，调用方分布在 asset-type-registry tests、snapshot/reflection 和 retained refresh；`snapshot/mod.rs` 还混入 transaction-history 重导出。因而继续扩 write scope 会把多个独立架构域机械并入 Editor09，不符合原子 ownership 与模块边界约束。

本轮按 `dev/UnrealEngine` 的 Content Browser 现源码重新对照整体算法，而不是依据局部实现猜测优化方向：`SAssetView` 明确区分 slow source refresh 与 quick frontend refresh；`ProcessItemsPendingFilter` 以 `MaxSecondsPerFrame` 分帧处理 pending filter；`HandleItemDataUpdated` 对 Added/Modified/Moved/Removed 做 in-place backend/filter 更新；`VisibleItems` 与 `RelevantThumbnails` 只跟踪可见项及邻近缩略图。Zircon current 的 chunked immutable generation、UUID/locator O(1) index 和 changed-chunk sharing 可作为正确的 source-generation 基础，但查询/filter 仍在同步 `build_snapshot` 中全量扫描，source/filter/sort/visible-paint 尚未形成 Unreal 式分层，且当前没有 1/1k/10k CPU、alloc、交互延迟或功耗数据。因此本轮不继续改缓存算法、不提交 managed validation、不声明性能收益；下一可执行步骤必须先由 asset-browser/current API 各 owner 集成或交付可转移的精确 parent 闭包，再在可运行 Editor 上采集 baseline 并写 hotspot report。

## 验证证据

- `rustfmt --edition 2021 --config skip_children=true --check`：两文件 exit 0。
- scoped `git diff --check`：exit 0。
- 行预算：root `721`，test owner `136`。
- current result SHA-256：root `63F97EE4BA3F299FAC7796D2A99FA665AD2234DF3FF044651A53E0E1466071FA`；test owner `AB0FC244CE5A3A114B97A2B94F24CA02B0885CC975FD1A175DFD112D9E56B11E`。
- `include_str!` 路径已按新 module topology 更新为 `../asset_workspace_state.rs` 与 `../../snapshot/data/editor_state_snapshot_build.rs`。
- 独立复审：Critical `0` / Important `2` / Minor `0`；外部 ABI closure 仍开放，上述 source guard 已按真实方法边界前向修复，未改 production 算法。
- 2026-08-31 current drift 复审：Critical `0` / Important `1` / Minor `1`；两条方法边界 guard
  均定位成功且计数各为 `1`，原确定失败已消失。剩余 Important 仅为 12 路径集合仍缺 modified
  `asset_browser.rs` 与其完整子树，不能形成 managed integration closure；Minor 为本行预算记录已从
  陈旧 `114` 更正为实际 `136`。
- `cargo test -p zircon_editor --lib --offline performance_tests` 在独立 D 盘 target 184.1 秒超时；复用 D 盘 product cache、限制 `-j 4` 后 304.4 秒再次超时。两次均未返回 Rust 诊断，第二次结束后确认无遗留 `cargo`/`rustc`/`link` 进程，因此状态是“未取得结果”，不是通过或代码失败。

## 产出记录与时间

| 时间 | 状态 | 完成项目与当前门禁 |
|---|---|---|
| 2026-08-29 00:53 +08:00 | `current-topology-modularized / owner-budget-restored / static-verified / external-abi-closure / known-current-test-failure` | 以精确 current-source SHA 为基线，将 `asset_workspace_state.rs` 的 5 个内联性能测试迁移到标准子模块，并加入 800 行结构守卫；production root 从 830 行降至 721 行，测试 owner 为 114 行。未复用归档 Editor09 blob，未改变生产算法或声称性能收益。rustfmt 与 scoped diff-check 均通过；两次 D 盘 focused Cargo 分别在 184.1 秒、304.4 秒无诊断超时。2026-08-31 current-topology 复审进一步确认三文件缺少 snapshot/type/re-export/consumer closure，且迁移前已存在的 lowercase source guard 会确定性失败，故 managed GREEN 仍待完整 current closure 与 profile-backed 算法决策，现有性能 failure 不关闭。 |
| 2026-08-31 03:41 +08:00 | `static-test-boundary-fixed / production-unchanged / static-verified / external-abi-closure / managed-pending` | 将 lowercase source guard 从“整文件全局计数”收窄为 `build_snapshot` 与 `patch_catalog_item_generation` 两个方法体分别计数，静态证明 full rebuild 与 catalog delta patch 各只规范化查询 1 次；未增加 query cache、未改 projection/patch 算法、未宣称性能收益。root hash 保持 `63F97EE4BA3F299FAC7796D2A99FA665AD2234DF3FF044651A53E0E1466071FA`，test owner 136 行、hash `AB0FC244CE5A3A114B97A2B94F24CA02B0885CC975FD1A175DFD112D9E56B11E`；rustfmt、scoped diff-check 与方法边界计数均通过。外部 snapshot ABI closure 仍阻止三文件独立集成，managed Cargo 仍 pending。 |
| 2026-08-31 04:59 +08:00 | `fresh-current-ownership / whole-module-reviewed / ue-reference-grounded / parent-closure-blocked / no-performance-claim` | 取消不可扩展的三路径 topology-review Session，并注册 12 路径 successor；通过 fresh fingerprint `e2ab8bc6cb5bb3f7634bc188f0fd99378fd279dc8daf2061d06831c6cfa280f7` 无 override 接管全部 current blob。随后完成 module declaration、public API consumer 与 Unreal Content Browser source/filter/update/visible-paint 分层审计，确认 12 路径仍不能 materialize 当前 parent：untracked logical paint 依赖 scope 外 modified `asset_browser.rs`，host/snapshot blob 还混入其它架构域。保留 chunked generation 方向，禁止在无 profile 数据时继续算法修改；managed validation 与 integration candidate 均未提交。 |
| 2026-08-31 07:31 +08:00 | `current-drift-rechecked / source-guard-fixed / parent-closure-still-blocked / record-corrected` | fresh plan-only owner `root-editor09-current-topology-record-refresh-20260831` 通过 transfer `1b51eef702754f18a6c7be797197d77e` 接管本记录，不接管 12 个混合源码 blob。独立复核 C0/I1/M1：rebuild 与 catalog patch lowercase guard 各计数 1，原确定失败已消失；`logical_paint_source.rs` 仍缺 scope 外 modified parent/subtree，禁止孤立 integration。更正证据段 test owner 行数 `114 -> 136`。 |
