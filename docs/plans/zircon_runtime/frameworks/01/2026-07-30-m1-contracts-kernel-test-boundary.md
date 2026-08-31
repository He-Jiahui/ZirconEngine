# Frameworks01 M1 contracts/kernel 测试边界硬切

> 承接 [`01-runtime-crate-decomposition.md`](../01-runtime-crate-decomposition.md) M1 Phase 1 的第一个物理迁移前置：`zr_contracts` 不得通过测试反向依赖 concrete `zr_kernel` 实现。

Plan: docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md

Milestone: M1

Status: source_implemented_static_green / state_kernel_owner_hard_cut_static_green / managed_rust_gate_blocked

Files: ["tools/tests/test_frameworks_01_contracts_kernel_test_boundary.py", "tools/framework_contract_partition_audit.py", "tools/tests/test_framework_contract_partition_audit.py", "tools/tests/test_frameworks_01_state_kernel_owner_boundary.py", "zircon_runtime/src/core/framework/foundation/config_manager.rs", "zircon_runtime/src/core/framework/foundation/config_manager_error.rs", "zircon_runtime/src/core/framework/foundation/mod.rs", "zircon_runtime/src/core/framework/scene/level_manager_error.rs", "zircon_runtime/src/core/framework/scene/mod.rs", "zircon_runtime/src/foundation/runtime/config_manager.rs", "zircon_runtime/src/foundation/runtime/config_manager/worker.rs", "zircon_runtime/src/foundation/runtime/config_manager_tests.rs", "zircon_runtime/src/scene/module/level_manager_contract.rs", "zircon_runtime/src/core/runtime/state_machine", "docs/plans/optimize/zircon_runtime/55/failure-2026-08-24-config-manager-domain-error-consumer.md", "docs/plans/zircon_runtime/frameworks/01/2026-07-30-m1-contracts-kernel-test-boundary.md", "docs/plans/zircon_runtime/frameworks/01/2026-08-24-m1-state-kernel-owner-hard-cut.md"]

## 范围

- contracts-side owner：`zircon_runtime/src/core/framework/render/environment/source_cubemap/tests/projection.rs`
- kernel integration owner：`zircon_runtime/src/core/runtime/tests/tasks.rs`
- boundary guard：`tools/tests/test_frameworks_01_contracts_kernel_test_boundary.py`
- contracts error owners：`foundation::ConfigManagerError` 与 `scene::LevelManagerError`；public
  trait 不得泄漏 `CoreError`，runtime implementation 在合同边界显式投影错误。
- dependency-complete source owner：`core/framework/render/environment/source_cubemap{.rs,/}` 的 projection/mipmap/PMREM 拆分。
- 不修改 production `ParallelSliceExecutor` 契约，不给未来 `zr_contracts` 增加 `zr_kernel` dev-dependency，不保留旧测试副本或兼容入口；并行 equirect sampler 直接硬切为真实并发所需的 `Fn + Send + Sync`。

## 当前状态

状态：`source_implemented / static_green_3_of_3 / partition_audit_schema2_green_18_of_18 /
state_kernel_owner_guard_green_6_of_6 / algorithm_partition_in_progress / wholesale_move_rejected /
runtime55_consumer_handoff_open / foreign_rhi_product_gate_blocked / managed_rust_gate_pending`

### 已完成项目

- 建立 production+test source 同口径静态守卫，扫描 `core/framework/**/*.rs`，禁止导入 concrete `crate::core::runtime` owner。
- TDD RED 为 `1 failed`，只报告 source-cubemap projection 测试的两处 `TaskPool` import（原第 74、88 行），与 M0 原子依赖基线完全一致。
- 原 r1/exact5 无法形成 clean-HEAD 原子提交：它依赖尚未提交的 source-cubemap tests/module 拆分。该会话已由 `frameworks01-m1-contracts-kernel-test-boundary-r2-20260730` 取代，失败的 validation-copy 尝试均停在 pre-Cargo 控制面阶段。
- r2 已领取并 attribution dependency-complete exact11：M1 guard/kernel tests 加上 archived Render13/Shader06 留下且无活动 owner 的 6 个 source-cubemap production/test dependency 文件；不吸收 importer、viewer、scene 或其他 Session 路径，也不宣称关闭 Render13 的更大 staging handoff。
- 两项 concrete `TaskPool` 集成测试已从 contracts-side source-cubemap test owner 移至 kernel-owned `core/runtime/tests/tasks.rs`，原位置不保留副本或兼容入口。
- `core/runtime/tests.rs` 已挂载新的 kernel integration test module；production `ParallelSliceExecutor` trait 未修改。并行 source-cubemap constructors 的 sampler bound 从串行 `FnMut` 硬切为 `Fn + Send + Sync`，与实际 face-parallel 调用一致，不保留旧 bound/shim。
- boundary guard 当前 `1/1` GREEN；snapshot 1354 已固定 exact11，首次独立复审为 `Critical 0 / Important 1 / Minor 0`，唯一 Important 即本记录的 stale r1/exact5/API 描述。
- Frameworks01 scene no-default-features job `d2bad3c6a3dc40d5860f11d1400003e9` 提供了 focused RED：kernel test 通过私有 `render::environment` 导入触发 E0603；公开 `framework::render` facade 已导出所需符号，最低修复不需要开放内部 module。
- kernel test 已改为从既有公开 `crate::core::framework::render` facade 导入，不公开 `environment` 内部模块、不新增兼容 re-export；source-cubemap 子树和 kernel test 叶文件已通过 Rust 1.94.1 scoped rustfmt。
- 最终静态证据为 boundary guard `1/1` GREEN、scoped rustfmt check GREEN、dirty exact9 `git diff --check` GREEN；snapshot 1354 已因本记录和 import/format 变化作废，不得复用。
- r2 的旧 wrapper 把 11 个路径错误登记为一个分号拼接且不可变的 `write_scope` 项。r2 已在保留源码和 review evidence 的前提下取消并释放 11 个租约；`frameworks01-m1-contracts-kernel-test-boundary-r3-20260731` 使用“父计划 + 编号子目录 + exact11”数组 scope 重新领取和 attribution 同一组哈希，不使用直接数据库修补或隐式 scope 扩张。
- snapshot 1356 对最终代码/文档内容的独立复审为 `Critical 0 / Important 0 / Minor 0`；r3 snapshot 1357 与 1356 的 11 个内容哈希逐项一致。本文状态更新后必须再生成 fresh r3 snapshot，1356/1357 都不得直接绑定 managed gate。
- M1 milestone prepare 的首次 wrapper 调用仅生成 action `37a6a05fba99442a9e9de7062d8668bd` 的未确认 preview，已自然过期且未生成 manifest；第二次受控 action `300a735047944a479dd954f680f2ce33` 明确失败为 `milestone_manifest_record_ambiguous`，原因是本文 `Plan:` 值包含 Markdown 反引号而不匹配 Session 的 canonical `plan_path`。现已按精确协议移除反引号；两次动作均未启动 validation/Cargo，不作为 managed gate 证据。
- 修正 `Plan:` 后的第三次受控 action `caa1a2f8feea42ca975464d8dae1abb9` 明确失败为 `milestone_manifest_not_attributed`：治理 scope 中的 `mipmap.rs` 与 `pmrem_layout.rs` 当前字节和 HEAD 完全一致，不是 dirty change。当前 milestone `Files:` 因此收敛为真实 dirty exact9；两文件仍由 r3 scope/lease 保护并由 baseline 自动进入 validation-copy，不制造空改动、不伪造提交归属。
- 2026-08-24 current-source 预检按“tracked 且存在 + nonignored untracked Rust 文件”口径得到
  `core/framework` 653 文件、80,477 行、2,684,368 bytes；path-sorted
  `path<TAB>file-sha256<TAB>bytes` manifest SHA-256 为
  `085c3af6ba6c86253213d216d248dba5c1d38b046487401eab428ee3cfd7b7c6`。旧的 tracked-only/
  missing-file 口径不得用于物理 `zr_contracts` manifest。
- 结构化 use-tree 审计确认 production contracts 中仅有两条 kernel error 反向边：
  `foundation/config_manager.rs` 与 `scene/mod.rs` 的 `crate::core::CoreError`；除此之外没有
  `Result<..., CoreError>` 合同。原守卫只匹配单行 `crate::core::runtime`，会漏掉 grouped use、alias、
  fully-qualified 与跨行 kernel error 路径。
- 守卫已复用仓库的 Rust literal/comment mask 与 nested use-tree parser。TDD RED 为 `1 failed`，精确报告
  上述 2 条 production 违规；scanner mutation 与 comments/literals negative control 为 `2/2` GREEN。
  生产硬切后的最终复跑在 3.306 秒内 `3/3` GREEN，scoped rustfmt 与 diff-check GREEN。
- `ConfigManager` 现在返回合同自有 `ConfigManagerError`，区分 runtime unavailable、persistence failure 与
  bounded flush timeout；worker 只有构造期 thread-spawn 仍在 implementation 内映射 `CoreError`。
  `LevelManager` 现在返回 `LevelManagerError`，区分 handle exhaustion、runtime/asset-manager/project
  availability、project-root mismatch、invalid locator、load/save/create failure；不提供 `From<CoreError>`、
  cross-type equality、alias 或 compatibility re-export。
- 参考引擎复核以 Unreal 为主：`FConfigCacheIni::Flush` 的成功/失败属于配置子系统，`UEngine::LoadMap`
  通过 World context 与显式 error 输出表达 map-load 失败；其边界不要求公共合同依赖生命周期 kernel error。
  Bevy/Fyrox 现有分域错误 owner 只作为交叉验证。本轮是 DAG/错误语义修复，不是性能优化，未进行或声明
  latency、throughput、allocation、功耗、同类引擎耗时或最优规模结论。
- 唯一 stale typed consumer 位于 Runtime55 活跃 owner 的 dirty `foundation/tests.rs`。Frameworks01 未改写
  该 blob，已建立 canonical handoff
  [`Runtime55 config-manager-domain-error consumer`](../../../optimize/zircon_runtime/55/failure-2026-08-24-config-manager-domain-error-consumer.md)，
  要求把断言硬切为 `ConfigManagerError::RuntimeUnavailable`，禁止兼容桥。
- `core/framework` 的 production-only 二级 core-path 审计覆盖 607 个文件、404 条引用：
  `framework` 197、`math` 158、`resource` 49，`manager/runtime/CoreError` 为 0。全 Runtime production
  domain audit 为 2,790 refs / 71 edges，`core/framework` 到 Asset/Graphics/Scene/Plugin/Builtin 的直接
  上行引用为 0。state hard cut 后的 fresh full Runtime audit 为 2,791 refs / 71 edges，禁止上行边和
  `rhi -> rhi_wgpu` 仍为 0；JSON 写入 `D:\zircon-frameworks01-current-domain-audit.json`，SHA-256
  `da7201bb5cee1bc969ff4ea6d6b122d2d8099d6eb0692d74784fd5f95d225a27`。
- 依赖重量审计没有发现 production contracts 引用 `wgpu`、`naga` 或 `glyphon`。2 个文件直接依赖中立
  `zr_rhi` 的 native-surface/UI-presenter contracts，方向无环且获准保留；词法 `image` 命中是本地
  `render::image` module，不是外部 `image` crate。
- 但是“纯 trait/DTO”假设本身不成立。原一次性词法统计的 3,418/2,606 函数与 207/385/15 文件分类
  没有稳定区分跨行函数、受限可见性与产品公开面，现已被
  `tools/framework_contract_partition_audit.py` 的 schema-1 报告取代。该工具复用 Rust comment/literal 与
  `cfg(test)` 屏蔽器，分类 mutation tests `5/5` GREEN；迁移前的可复现基线为 607 production 文件、
  52,541 行非空 production code、3,634 个函数体、2,542 个产品 `pub` 函数体、176 个受限可见性函数体、
  48 个 public trait，文件分类为 205 declaration-only、380 mixed、22 behavior-only。
- 第一项 physical partition 已把 12 文件/519 行 runtime-wide state machine 整体硬切到
  `core/runtime/state_machine`，未来随 `zr_kernel` 迁移；旧 `core/framework/state` 与 10 处旧 consumer
  均归零；chained module alias mutation RED→GREEN 后 owner guard `5/5` GREEN。迁移后当前基线为
  595 production 文件、52,108 行、3,579 个函数体、
  2,532 个产品公开函数体、144 个受限可见性函数体、47 个 public trait，分类为
  204 declaration-only、369 mixed、22 behavior-only。完整记录见
  [`state kernel owner hard cut`](2026-08-24-m1-state-kernel-owner-hard-cut.md)，D 盘 JSON SHA-256 为
  `a6b9712bc02c32f3d0c0b394c5b8276cf786c322ddf2b68d0b16f1fa66bd514d`。
- Unreal-primary 对照进一步否定 wholesale move：local source 将 RHI/RenderCore/Renderer 分为
  101/226/792 个源文件，后处理与 reflection-environment 算法位于 Renderer 而非通用 contract owner；
  Bevy 将环境光/PBR 行为放 `bevy_pbr`、设备/资源行为放 `bevy_render`，Fyrox 使用独立
  `fyrox-graphics`。因此先做 declaration/behavior physical partition，再建立 `zr_contracts`；不得用 feature
  flag 把实现算法藏进合同 crate，也不得为保持旧目录批量 re-export。
- schema-1 分类器的 current-source 完整 `cProfile` 基线为 33.447 秒 / 4,431,593 calls；其中
  `_test_only_source_paths` 为 28.742 秒、扫描 10,194 个 Rust code view，目标却只有 761 个
  `core/framework` production 文件。它还会把函数局部 struct/enum/type、trait/impl associated type 和
  `macro_rules!` 模板当作物理合同声明或函数实现，因此不能作为 hard-cut item manifest。
- schema-2 改为只读取 `core/framework` 物理 inventory，并在该子树内传播 `cfg(test)` module reachability；
  无关 `graphics` 文件即使不可解码也不再影响报告。结构化分类区分 free/impl/trait-default/nested 函数体，
  排除函数局部类型和宏模板膨胀，单列 impl block、static storage 与宏生成面。TDD RED 为 4 failures / 2
  errors；profile 后的 read-once regression 与 const-generic return-header regression 各以 `1 failure`
  证明 eager-default 二次读和 `{ ... }` 函数体起点误判；独立 review 的 external/item/expression bang-macro
  三项用例以 `3 failures` 证明依赖宏 token tree 会膨胀声明/函数计数。最终
  mutation/negative/per-file-read/macro controls 全部 GREEN；review 后补充的 test-only module
  传递传播与 production/test 双路径共享回归直接 GREEN，最终为 `18/18` GREEN。
- Unreal-primary 复核进一步修正了“impl 或 public 文件内函数体都应迁出”的错误方向：
  `Core/Public/Templates/UnrealTemplate.h` 保留 constexpr/inline 值语义，
  `Core/Public/Misc/ConfigCacheIni.h` 保留局部 inline invariant/accessor，
  `RenderCore/Public/RenderGraphResources.h` 保留资源合同上的 inline access/validation，而大规模渲染算法仍由
  `Renderer/Private` 拥有。因此 Rust associated const/marker impl 属于 contract binding，维护 DTO 自身不变量的
  轻量方法允许留在合同 owner；只有编译、调度、I/O、全局状态、GPU/场景执行等实质行为进入迁移候选。
- 宏生成面不再被工具猜测为 behavior：当前 `net/ids.rs` 与
  `window/window_state/generation.rs` 生成的是 DTO/代际值对象，schema-2 将两者标记为
  `macro_generated_review` + `manual_review_required`，必须由展开后的 public-item manifest 和 owner review
  判定，不允许词法分类器自动移动。所有 path-qualified/raw bang-macro invocation token tree 在结构统计前被
  屏蔽；当前 10 个 item invocation 全部落在上述 2 个文件，函数体内 expression macro 不扩大 review 集合。
- stable HEAD `a48446132f1f4f3b55dbca364c23a43067ad452f` 的 schema-2 current report 连续三次均为
  761 files / SHA-256 `3058fa36ed6d16463be1451d2f69a6d48ef190d2cbb55c17e0c86d94886a689c`，
  最终 read-once/const-generic/macro 版本用时 10.859/10.739/8.935 秒，中位数 10.739 秒。完整
  `cProfile` 为 8.709 秒 / 1,657,107 calls，相对 schema-1 profile 分别下降 74.0% / 62.6%；
  宏候选 presence gate 将 bang-macro scanner 从 1.665 秒降至 0.695 秒；`_rust_code_view` 调用
  10,194 -> 1,589，文件打开 7,737 -> 823。产物位于
  `D:\zircon-framework-contract-partition-schema2-final.{json,prof}`，未写入 C 盘。
- current schema-2 基线为 761 production files / 56,693 non-empty lines / 3,945 function bodies：
  261 declaration-only、462 mixed、36 behavior-only、2 macro-generated-review；另有 1,105 impl blocks、
  661 free / 3,243 impl / 37 trait-default / 4 nested function bodies、9 static items。该数字是分区复核优先级，
  不是“462 个文件必须全部迁出”的结论，也不代表产品运行时性能或功耗数据。
- 2026-08-27 对当前 `HEAD=7583bd4998bdd2fac73ad6382e434543e0429082` 与共享 dirty tree 重新执行
  schema-2：分类器单测 `18/18` GREEN（0.499 秒），current JSON 写入
  `D:\zircon-framework-contract-partition-schema2-current-20260827.json`，SHA-256 为
  `d59fdc20bf0536b0edf6374d056f43ece292178aa423b3b6f01882d2691e5188`。当前报告为 766 files /
  57,504 non-empty lines / 3,992 function bodies，分类为 261 declaration-only、467 mixed、36
  behavior-only、2 macro-generated-review；相对 stable snapshot 新增 5 个文件、20 个既有 row 发生内容统计变化、
  0 个文件删除。该结果证明工具仍能追踪移动中的 current source，但共享 dirty tree 不是 immutable milestone
  baseline，不能覆盖前述 stable HEAD 产物或被表述为三次稳定冻结。
- 用 schema-2 与 [`engine-code-structure-convention.md`](../../../engine-code-structure-convention.md) 的 R1.1
  交叉检查后，`core/framework` 当前只有两个 `mod.rs` 仍含 production function body：
  `navigation/asset/mod.rs`（15）与 `bridge/mod.rs`（1）。前者同时拥有 DTO、构造、bincode codec、v1 migration
  与 debug projection，存在广泛 Runtime/Navigation plugin consumer，必须按 contract / construction / codec /
  debug projection owner 做原子 hard cut，不能只把 286 行均分成叶子；后者的唯一行为是 generation/provider
  到 `BridgeInterfaceStatus` 的映射，真正状态 owner 是 plugin bridge table。
- Unreal-primary 复核 `Core/Public/Features/IModularFeatures.h` 与
  `Core/Private/Features/ModularFeatures.{h,cpp}`：公共面保留 feature interface/lookup contract，注册表、锁、
  provider availability 与 lifecycle mutation 由 private implementation 持有。该边界支持把 Zircon 的
  bridge interface/status DTO 留在 neutral contract owner，而把 generation parity/provider-installed 状态推导
  放回 `plugin/bridge/table` implementation；不支持把 table lifecycle 行为留在未来 `zr_contracts`，也不支持
  为维持旧内部路径增加 compatibility re-export。当前 `bridge/mod.rs` 不在本 Session immutable scope，
  `plugin/bridge/table.rs` 又含外来未提交性能 blob，因此本轮只记录下一原子批次，不越权改写。

### 待完成项目

- 等待 Runtime55 返回唯一 typed consumer；在此之前 source hard cut 已实现，但完整 Rust test target 预计仍会
  在该 foreign assertion 编译处失败。
- 按 schema-2 复核集合与 owner 协调选择原子批次：优先 render/animation/compiler/picking 中具有编译、调度、
  I/O、全局状态、GPU/场景执行的实质行为；不得按 mixed 数量批量移动轻量 DTO invariant/accessor、associated
  const 或 marker binding。每批用结构化 public-item manifest 证明 descriptor/trait/error 身份不漂移、旧实现
  路径和重复算法为 0；宏生成文件还必须验证展开后的 public API。此分区完成前禁止创建半空 `zr_contracts`。
- 刷新本文更新后的 immutable snapshot/attribution，复核独立 `C0/I0/M0` 结论并执行 fresh
  validation-copy focused Rust gate；旧 materialization 和 snapshot 1354/1356/1357/1360/1361 均不得在源变更后复用。
- managed Rust GREEN 后通过 coordinator 原子提交，并把真实 job/run/结果写回本记录；在此之前不声明 Rust gate 通过。
- 完成后刷新 Frameworks01 M0/M1 状态；在此之前不声明 `zr_contracts` 创建前置完成，也不开始物理 crate move。
