# Render06 ViewFamily 物理所有权硬切记录

## 会话与范围

- 日期：2026-08-24
- 会话：`render06-view-family-owner-split-r1-20260824`
- 父计划：`docs/plans/zircon_runtime/render/06-temporal-pipeline.md`
- 当前状态：实现完成、静态验证完成、受管动态验证被共享 validation-copy baseline drift 阻塞（Cargo 未启动）
- 提交状态：未提交；尚未达到里程碑提交或企微通知条件
- 变更范围：
  - 删除 `zircon_runtime/src/core/framework/render/view_family.rs`
  - 新增 `zircon_runtime/src/core/framework/render/view_family/mod.rs`
  - 新增 `zircon_runtime/src/core/framework/render/view_family/resolution.rs`
  - 新增 `zircon_runtime/src/core/framework/render/view_family/dynamic_resolution.rs`
  - 新增 `zircon_runtime/src/core/framework/render/view_family/pipeline.rs`
  - 新增 `zircon_runtime/src/core/framework/render/view_family/tests.rs`

## 架构复核

父计划明确要求将单体 `view_family.rs` 硬切为 `view_family/{resolution,dynamic_resolution,pipeline,tests}.rs`，并让根模块只保留声明与显式导出。本切片不改变动态分辨率算法、时序重建策略或帧提交行为，只收敛物理所有权，避免 Render07/Render17 继续向 1381 行单体文件叠加职责。

主要参考为 Unreal Engine：

- `dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/SceneView.h` 将 screen-percentage/upscaler 接口与 `FSceneViewFamily` 的聚合生命周期分开表达。
- `dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PostProcess/PostProcessing.cpp` 区分 PrimaryUpscale、SecondaryUpscale，以及 PrimaryToSecondary、PrimaryToOutput、SecondaryToOutput 阶段。

因此 Zircon 的拆分边界按“分辨率策略与值对象 / 动态分辨率反馈控制 / ViewFamily 阶段拓扑与几何解析 / 回归测试”划分，而不是按文件行数机械切块。

## 已完成项目

- `mod.rs` 仅保留 4 个子模块声明和显式 `pub use`，未增加 wildcard facade 或兼容路径。
- `resolution.rs` 负责分辨率常量、策略、计划、目标、阶段目标、时序历史键、upscaler 和 fraction 规范化。
- `dynamic_resolution.rs` 负责 controller、scope、sample、decision 与反馈收敛逻辑。
- `pipeline.rs` 负责阶段拓扑、ViewFamily pipeline 解析和 viewport/allocation 几何辅助逻辑。
- `tests.rs` 承接原有 17 个测试；测试名称集合无增删。
- 删除零调用的私有 `scale_extent` 辅助函数；没有保留旧文件或兼容重导出。

## 量化结果

| 项目 | 拆分前 | 拆分后 |
|---|---:|---:|
| 生产/测试单文件行数 | 1381 | `mod.rs` 18、`resolution.rs` 223、`dynamic_resolution.rs` 261、`pipeline.rs` 437、`tests.rs` 458 |
| 最大文件行数 | 1381 | 458 |
| 对外公开符号数 | 15 | 15 |
| 测试名称数 | 17 | 17 |

公开符号集合保持为：`MIN_RENDER_RESOLUTION_FRACTION`、`MAX_RENDER_RESOLUTION_FRACTION`、`RenderResolutionPolicy`、`RenderDynamicResolutionController`、`RenderDynamicResolutionScope`、`RenderDynamicResolutionGpuSample`、`RenderDynamicResolutionDecisionReason`、`RenderDynamicResolutionDecision`、`RenderResolutionPlan`、`RenderViewFamilyTarget`、`RenderViewFamilyPhaseTargets`、`RenderTemporalHistoryKey`、`RenderUpscalerKind`、`RenderPipelinePhase`、`RenderViewFamilyPipeline`；静态集合对比未发现新增或遗漏。

## 验证证据

- `rustfmt --edition 2021 --check`：5 个新文件通过。
- 路径限定 `git diff --check`：通过。
- 公开符号集合对比：通过，原集合与新集合一致。
- 测试名称集合对比：通过，17/17 一致。
- 声明契约二次复核：发现并修复首轮拆分中 Controller、ResolutionPlan、PipelinePhase 的 doc/derive 附着错误；修复后 13 个公开 struct/enum 的派生集合逐项与 HEAD 一致，生产文件无错误附着到函数的 derive。
- 生产代码 panic 类扫描：`unwrap`、`expect`、`panic!`、`todo!`、`unimplemented!` 零命中；测试断言中的 `expect` 保持不变。
- 结构审计：`audit_runtime_structure.py --json` 完整运行结束，原 `view_family.rs` 与新拆分文件均未出现在 `large_file_hotspots`；所有新文件均小于 800 行，原 1381 行大文件热点被消除。
- 全局边界：同次审计的 `module_convention_gate.m1_gate_status` 仍为 `migration-debt-present`；本切片只关闭 ViewFamily 物理所有权热点，不宣称仓库整体 M1 已转绿。

### 2026-08-24 current-source 漂移复核

- 主线已从本会话基线 `8dc299a8b65813f692e222a709f951e6ace90be6`
  推进到 `16122ac757cf3b2e60e43477bda6c5fa94c63ddb`；两个 HEAD 中旧
  `view_family.rs` 的 Git object 均为
  `da91e2ec6503a04598d11b87d539bc1a96e63ccc`，被拆 owner 没有发生源码漂移。
- 5 个新源文件的 Git object 依次为
  `e90d5a80408c16bd84d3c2230a80401d75877ada`、
  `e05cd33ee02beb3347561fd6ae278b14d8c55433`、
  `a9df4c603d50a2c2e8cb1adb97644dbcead76a1f`、
  `3095823a7279051ae85619e1e5bd2524c5542558`、
  `02aa83fdcfb2079777f816d29a8ad7fdba467d9a`；重跑
  `rustfmt +1.94.1 --edition 2021 --check` 通过。
- 对当前 HEAD 重做公开符号与测试名集合对比，仍为 15/15 与
  17/17；唯一移除的函数仍是零调用私有 helper `scale_extent`。
- 六个外来 staged-add 路径仍是 `AM`，因此不提交第二张同构
  validation ticket；本次是 current-source 静态复核，不是 Cargo 或性能验收。

## 待完成与边界

- 受管验证 ticket `b600c1e659414a2088d1d6a6799b1362` 冻结了 5 个新增源码 SHA-256 和旧 `view_family.rs` 删除 tombstone，目标命令为 Windows Rust 1.94.1 下的 `zircon_runtime` `core-min` ViewFamily 17 个库测试。
- 该 ticket 在 Cargo 前以 `validation_copy_baseline_drift` 终止，阶段为 `materialization_prepare`；验证副本 job 为 `d0fe340d4bd84011a75864b93c7e5af4`，没有生成编译或测试结果。
- fail-closed 路径与共享阻塞指纹一致：
  - `zircon_runtime/crates/zr_rhi_wgpu/src/tests/device_contract/transfer_and_submissions.rs`
  - `zircon_runtime/src/ui/surface/binding_targets.rs`
  - `zircon_runtime/src/ui/template/asset/compiler/binding_param_resolver.rs`
  - `zircon_runtime/src/ui/template/asset/compiler/control_scope.rs`
  - `zircon_runtime/src/ui/tests/asset_binding/control_scope.rs`
  - `zircon_runtime/src/ui/tests/asset_prototype_store/control_scope.rs`
- 这些路径不属于 Render06 immutable scope，本会话不认领、不改写、不绕过 fail-closed；在归属修复前不盲目提交第二张同构 ticket，也不据此宣称 GREEN。
- 本切片没有性能剖析或功耗测量，因为没有改变算法和执行热路径；不得从文件拆分推导帧耗、功耗或算法最优性结论。
- 在受管动态验证通过并完成独立复核前，不更新父计划完成状态，不创建里程碑 commit，不发送企微完成通知。
