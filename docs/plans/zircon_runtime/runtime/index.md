---
related_code:
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/builtin/runtime_modules.rs
  - zircon_runtime/src/core/mod.rs
  - zircon_runtime/src/scene/ecs/mod.rs
  - zircon_runtime/src/plugin/mod.rs
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
  - zircon_app/Cargo.toml
  - zircon_runtime/Cargo.toml
  - dev/bevy/crates/bevy_app/src/main_schedule.rs
  - dev/bevy/crates/bevy_asset/src
  - dev/Fyrox/fyrox-impl/src/plugin/mod.rs
  - dev/UnrealEngine/Engine/Source/Runtime
plan_sources:
  - .codex/plans/Zircon Runtime 架构渐进式 Review 与优化计划.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
  - docs/plans/zircon_runtime/render/index.md
---

# Zircon Runtime 架构完善与优化总体计划

本目录是 `zircon_runtime` 在渲染骨架计划（`docs/plans/zircon_runtime/render/01-08`）之外的 runtime 侧架构完善总计划：技术选型治理、core spine 与 root surface 收束、调度/帧循环、资产管线、scene/editor 边界收尾、插件公开面与生命周期、runtime 侧性能热路径。它承接 `.codex/plans/Zircon Runtime 架构渐进式 Review 与优化计划.md`（M0–M7），把其中 runtime 侧尚未完成的项落成可执行子计划，并以 2026-06-12 的实仓审计修正其现状假设。

## 1. 技术选型评审结论（2026-06-12 实仓核对）

总体结论:选型与 Bevy 同代生态一致（winit + wgpu/naga + glam + crossbeam + notify + taffy），自研 ECS/UI/资产管线的目录形状与 `bevy_ecs`/`bevy_asset` 可逐项对照，方向合理。但"声称技术栈"与实仓有 5 处失实，另有 4 个能力缺口需要决策。

### 1.1 声称栈核对表

| 声称 | 实仓状态 | 证据 | 评估 |
|------|---------|------|------|
| winit 窗口输入 | ✓ `0.31.0-beta.2`，optional + feature 门控 | 根 `Cargo.toml`、`zircon_runtime/Cargo.toml` | 合理；**beta 版本风险**，见子计划 01 |
| wgpu / naga 渲染 | ✓ 29.0.1 / 29.0.1 配套一致 | 根 `Cargo.toml` | 合理 |
| taffy UI 布局 | ✓ 0.10，自研 UI 经 `ui/layout/taffy_bridge.rs` 桥接 | `zircon_runtime/Cargo.toml` | 合理 |
| glam 数学 | ✓ 0.32.1 (serde) | 根 `Cargo.toml` | 合理 |
| fontdue / cosmic-text / glyphon | **部分失实**:fontdue 0.9.3 仅在 editor;cosmic-text 不存在;runtime 实际为 glyphon 0.11 + fontsdf 0.5.3 + 自研 text shaper | `zircon_editor/Cargo.toml`、`zircon_runtime/Cargo.toml:77-78` | 三库职责口径需定稿，见子计划 01 |
| 自研 ECS | ✓ `scene/ecs/`（archetype、query、schedule、parallel executor、conflict graph、observer、change detection） | `zircon_runtime/src/scene/ecs/` | 形状对齐 `bevy_ecs`，合理 |
| kira 音频 | **失实**:不存在;实际为 cpal 0.15（optional）+ sound 插件自研 DSP/HRTF/occlusion/mixer | `zircon_plugins/sound/runtime/Cargo.toml` | 既有自研混音栈下 cpal 底座更合理，**不建议**再引 kira;矫正文档即可 |
| image / serde / crossbeam 异步加载 | ✓ 全部存在;worker pool + watcher 在 `asset/pipeline`、`asset/watch` | `zircon_runtime/src/asset/` | 合理 |
| zip / tar 打包 | **失实**:均不存在;仅 zstd 0.13.3;`ExportPackagingStrategy` 已有契约但无归档实现 | `zircon_runtime/Cargo.toml:100`、`plugin/export_profile.rs` | **导出打包能力缺口**，见子计划 01 |
| gilrs 手柄 | ✓ 0.11.0，optional，`gamepad-gilrs` feature | `zircon_app/Cargo.toml:84` | 合理 |
| tracing / tracing-subscriber | ✓ tracing 常驻;subscriber 0.3.20 仅在 `profiling-tracy` 后 | `zircon_runtime/Cargo.toml:24,93` | 合理;profiling 构建超时问题见子计划 07 |
| rfd / arboard 编辑器辅助 | **失实**:均不存在 | 全仓 Cargo.toml grep 无命中 | editor 侧文件对话框/剪贴板缺口，归 `zircon_editor` 决策 |

### 1.2 声称栈未列、但实际承重的依赖

rayon（ECS/资产并行）、tokio + hyper + reqwest + tokio-tungstenite（net 插件网络栈）、bincode/ron/toml/serde_json（多格式序列化）、gltf/tobj/dxf/ply/stl（模型导入）、notify 9.0.0-rc.3（资产监视）、libloading（cdylib runtime 与 native 插件）、zstd、accesskit（optional）、Recast C++ 经 cc 绑定（navigation 插件）、tauri 2.11（zircon_hub，**Slint 已不在任何 Cargo.toml**）、zr_vm_rust_binding（**指向仓库外 `../../zr_vm` 的路径依赖**）。

### 1.3 版本与依赖治理风险

1. `winit 0.31.0-beta.2`:beta 跟踪策略未定稿（锁定/升级 gate）。
2. `notify 9.0.0-rc.3`:RC 版本。
3. `zr_vm_rust_binding` 路径依赖逃逸出仓库根，影响 clone 即建的可复现性（optional 缓解，仍需文档化或 vendor 决策）。
4. `jolt = []` 空 feature + `zircon_plugins/physics` 空壳:**物理引擎完全未决**（jolt-rust / rapier / 自研三选一），是完备性最大缺口。

## 2. 架构评审结论

### 2.1 已收敛项（旧计划假设需修正处）

旧《渐进式 Review 计划》的部分目标已经落地，本计划不再重复:

- `zircon_runtime/src/lib.rs` 仅 75 行，root surface 已薄化;`builtin/runtime_modules.rs` 已是 28 行 folder-backed 装配入口（assembly/manifest/core_modules/plugin_modules/load_report 分离）。旧计划 M1/M2 对这两处的"待拆"描述已过时。
- `zircon_app` 插件扇出已收束为单一 `zircon_first_party_runtime_catalog` optional 依赖（`zircon_app/Cargo.toml:88`），feature 全部经 catalog 转发。
- 服务注册热路径已是强类型键:`HashMap<RegistryName, ServiceEntry>`，`RegistryName` 带缓存解析偏移（`core/runtime/descriptors/registry_name.rs`），非 per-frame 字符串查找。
- World 序列化不含 selection/viewport authoring 状态，且已有 serialization_guard 守卫;测试树全部 folder-backed，无巨型 `tests.rs`。
- 渲染分层（framework 契约 / graphics 实现 / rhi / rhi_wgpu）与 UE 的 Engine/Renderer/RenderCore/RHI 方向一致，骨架差距已由 render 子计划 01–08 接管。

### 2.2 问题清单（本计划的工作对象）

| # | 问题 | 证据 | 子计划 |
|---|------|------|--------|
| P1 | `lib.rs:39-72` 的 70+ 类型 `pub(crate) use` 巨型别名块:内部调用方经 crate 根访问 graphics 内部类型，掩盖 owner 路径;且混入 HybridGi/VirtualGeometry/Solari 等插件 provider 语义 | `zircon_runtime/src/lib.rs:39-72` | 02 |
| P2 | `core/` 根下 13+ 散件（channel_util、config_store、event_bus、frame_clock、job_scheduler、lifecycle、state、tasks、time、types、error、diagnostics、modules）游离于五件套 spine 口径之外，归属未定 | `zircon_runtime/src/core/` 目录 | 02 |
| P3 | ~~帧循环阶段无单一权威定义~~（2026-06-12 M0/M1/M2 复核矫正:`SystemStage` 九阶段权威表、`FIXED_LOOP`、Real/Virtual/Fixed 三时钟均已落地）。`WorldDriver` 已改为消费上游 `RuntimeTimeAdvance`，二次 `advance_time_by(...)` 与 driver 局部 cap=4 已删除；`FixedStepPlan::overstep_fraction()` 已提供；UI extract 已定稿为 runtime 03 合法 dynamic-session side path；阶段内顺序已有注册顺序负例守卫。M3.1 已补 `ScheduleParallelExecutionReport`、关闭并行开关与 `schedule.parallel_batches` / `schedule.serial_fallbacks` 诊断；M3.2 已补代表性多批次收益与串并行终态一致性守卫。剩余缺口:Cargo 回归待运行 | `docs/zircon_runtime/core/frame_schedule.md`、`docs/zircon_runtime/scene/ecs/schedule_parallel_executor.md`、`scene/ecs/system_stage.rs:4-45`、`core/runtime/time.rs`、`core/framework/time/fixed_step_plan.rs`、`scene/module/world_driver.rs:11-73`、`dynamic_api/session.rs:548-553`、`scene/ecs/schedule_parallel_executor.rs`、`scene/tests/ecs_schedule.rs`、`dynamic_api/tests/session_lifecycle.rs` | 03 |
| P4 | `plugin/mod.rs:27-50` 把 native plugin loader 约 48 个类型/常量与 ABI V1/V2/V3 三代全量 `pub`，与"native loader 退出公共主路径"决策冲突;细化重核:V1/V2 真实使用者仅 `native_dynamic_fixture` 夹具 1 文件，export 宿主文件不引用版本符号，淘汰面比预估小 | `zircon_runtime/src/plugin/mod.rs:27-50` | 06 |
| P5 | 实测性能 ~10fps（230 draws / 231 pre-draw buffer copies / 31 passes）;权威 FPS 取证被 ZrVM native assertion（`zr_vm_core.dll function.c:1394`，空参数空指针）阻塞;2026-06-12 binding 层已修复空参数 marshalling 并通过 binding 回归，runtime 真实后端验证仍待完成;profiling 构建两次超时 | `.codex/sessions/20260611-0416-rendering-10fps-analysis.md` | 07（render 侧归 render/01-08） |
| P6 | ~~`scene/editor_projection/` 空目录残留~~（2026-06-12 矫正:目录已删，残留文本只保留在禁止复活守卫中）;runtime 内 "editor" 白名单裁决与守卫机器化已落 `runtime_naming_boundary` / `runtime_absorption::naming_boundary`（editor 1260 locations / 0 unclassified；序列化纯净 token 守卫已存在:`scene/tests/authoring_boundary.rs` 双 token 表 19+25 词） | `scene/tests/authoring_boundary.rs`、`scene/tests/inspection.rs`、`runtime_absorption/naming_boundary.rs` | 05 |
| P7 | 资产管线对照差距:细化重核矫正——强类型 `Handle<TAsset>`、五态 `AssetLoadState`×3 级、事件族（含 ReloadFailed + revision）、120ms 去抖均已存在;真实缺口收窄为:句柄无引用计数语义（裁决）、worker pool 无去重/背压（unbounded 实测）、watcher 行为测试缺失 | `asset/facade/{handle,load_state,event}.rs`、`asset/pipeline/worker_pool.rs:20-21` | 04 |
| P8 | generated 口径矫正:真实文件头 `// @generated ...` 标记为 0，"generated" 词根命中 42 文件（混杂法线生成等领域词）;标记规范与 leaf DTO/table 守卫已固化。M4.2 已把生成入口中的 direct `EntryRunner`/`NativePluginLoader` 行为迁入 `zircon_app::entry::export_bootstrap`，并把 linked plugin registration 从即时调用改成 app provider 表；provider 行已裁决为允许的 generated table adapter，结构审计从 13 行为点/5 标签/5 债务降到 6 行为点/3 标签/0 债务，`m1_gate_status=classified-and-clear`；Cargo 测试阶段仍受 render/graphics 编译漂移与 Windows 测试目标超时影响 | `docs/engine-architecture/generated-code-boundary.md`、`zircon_app/src/entry/export_bootstrap.rs`、`plugin/export_build_plan/*template*.rs` | 02 |
| P9 | "legacy" 命名已机器化裁决（runtime-only 审计 516 locations / 0 unclassified）；剩余 10 个 debt bucket 按 runtime UI input/render、graphics、DDS、UI template/layout、input、asset、dynamic API、scene schema owner 分派 | `runtime_naming_boundary` 结构审计 | 05 |
| P10 | 物理空壳、导出归档（zip/tar）、editor 辅助（rfd/arboard）三个完备性缺口未决策 | §1.3 | 01 |
| P11 | ECS 数据面与 `bevy_ecs` 对照已启动（2026-06-12 盘点修正）:`StorageType::{Table,SparseSet}` 与 `ComponentStorage` 双 backing store 已确认；实体生命周期测试矩阵、观察者三类入口触发时机测试、命令队列错误报告面、events/messages 双通道分工测试、change tick 回绕与 stale 窗口截断测试已落地待 Cargo | `docs/zircon_runtime/scene/ecs.md` Runtime 08 数据面对照、`scene/ecs/` 40 条目盘点、`observer.rs:54-112`、`commands/command_queue.rs`、`events.rs`、`messages.rs`、`change_detection/{change_tick.rs,change_tick_window.rs}` | 08 |
| P12 | UI 子系统:`ui/v2/` 与非 v2 双代并存定位未裁决;输入路由（input/pointer/navigation/dispatch）单点权威未声明，05 审计的 UI legacy debt 三桶待承接;taffy 直连面未收敛到 bridge 单点 | `ui/` 17 条目、`surface/` 21 条目盘点、`surface/input/navigation.rs:22-54` | 09 |
| P13 | cdylib 函数表版本不同步（`ZrRuntimeApiV1`/`ZrHostApiV1` vs plugin 宿主 `ZrHostApiV3` + 子 API V1×4）已由 runtime 10 定稿保守 bump 规则;interface `ui/` 22 条目镜像契约无漂移守卫;session 失败路径中的坏句柄/缺失非零句柄/销毁注册表 removal 守卫已 scoped Cargo 通过，`minimal`/`headless` profile 已改为跳过 `RuntimeRenderBridge` 并补 live double-destroy / destroyed-handle 夹具；该 headless lifecycle slice 已静态通过，Cargo 验证在活动 `zircon_runtime` 编译通道下 904s 超时，待空闲窗口重跑 | `runtime_api/api_table.rs:43,63`、`plugin_api.rs:41-227`、`dynamic_api/exports.rs:25`、`dynamic_api/session.rs`、`dynamic_api/tests/session_lifecycle.rs` | 10 |

### 2.3 参考引擎证据锚点

| 维度 | Bevy | Fyrox | Unreal |
|------|------|-------|--------|
| 帧循环权威 | `dev/bevy/crates/bevy_app/src/main_schedule.rs`（MainScheduleOrder） | `dev/Fyrox/fyrox-impl/src/engine/executor.rs`（fixed-step 累积） | Tick group（PrePhysics→PostUpdateWork） |
| 渲染世界分离 | `dev/bevy/crates/bevy_render/src/extract_plugin.rs`（RenderApp SubApp + ExtractSchedule） | `fyrox-impl/src/renderer/` | RHI/RenderCore/Renderer 三层 |
| 资产 | `dev/bevy/crates/bevy_asset/src/{loader.rs,handle.rs,server/,processor/,meta.rs}` | `dev/Fyrox/fyrox-resource/src/{manager.rs,loader.rs,state.rs,event.rs}` | AssetRegistry |
| 插件/热重载 | Plugin/PluginGroup（静态） | `fyrox-impl/src/plugin/{mod.rs,dylib.rs}`（DynamicPlugin + 状态序列化重载） | 模块系统 |
| 模块尺度 | crate-per-subsystem（约 40+ crate） | crate-per-layer（core/resource/graph/ui/impl/editor） | `Engine/Source/Runtime` 约 189 模块，Runtime/Editor/Developer/Programs 四分 |

`zircon_runtime` 取"单 crate + 内部 spine"形状，介于 Fyrox（多 crate 分层）与 Unreal（巨型模块树）之间;在 cdylib 热重载约束下单 crate 是合理选择，但要求内部模块边界承担 Bevy 中 crate 边界的职责——这正是 P1/P2/P4 要修的内容。

## 3. 子计划地图与执行顺序

| 计划 | 文档 | 依赖 | 状态 |
|------|------|------|------|
| 01 技术选型与依赖治理 | `01-tech-stack-and-dependency-governance.md` | 无 | in_progress（M1 runtime-tech-stack 文档、架构索引挂接与 tech_stack 依赖守卫已落地，standalone rustc 守卫 10/10 通过；M2.1/M2.2 text.md 三层文本栈矩阵、tech-stack 交叉引用、NativeGlyphon 状态口径与 complex text 候选裁决守卫完成，cosmic-text/Parley/Swash/HarfBuzz 只能经替换 `UiTextShaper` 接入；M2.3 fontdue 裁决为 editor retained-host 临时文本 fallback；M3.1 物理后端选型文档完成，builtin 为 V1 唯一可执行后端、Jolt 为未来 native 方向但保持 unavailable；M3.2 导出归档决策完成，未来桌面/editor 容器选 ZIP 但当前不引入 zip/tar 依赖；M3.3 editor-only 依赖 backlog 完成，rfd/arboard 不进 runtime/interface；2026-06-13 Cargo `tech_stack` 重跑未执行到测试，lib-test 编译先被活跃 plugin bridge 切片的 `extension_registry_bridge.rs` unresolved `BridgeInterfaceSnapshot` / `BridgeInterfaceStatus` 导入阻断） |
| 02 core spine 与 root surface 收束 | `02-core-spine-and-root-surface.md` | 无 | in_progress（M1/M2.1/M2.2 物理 cutover 完成；root/naming 独立守卫通过；M4.1 generated 规范与裁决完成，M4.2 generated 结构守卫与 app export-bootstrap owner 已落地，export 模板 direct native-loader/EntryRunner/registration-call 行为已迁出，provider 表裁决为允许的 generated table adapter，结构审计 0 迁移债且 `classified-and-clear`；M2/M4 全量 Cargo 回归当前受 render/graphics 活动编译与测试目标超时影响，M3 lib.rs graphics alias 清理需等待 render owner 稳定） |
| 03 调度与帧循环对齐 | `03-schedule-and-frame-loop-alignment.md` | 02（core 归属定稿） | in_progress（M0/M1/M2/M3 实现与源守卫已落地；Cargo 回归被无关 `ui/tests/asset_dependency_index.rs` 导入错误阻断，待重跑） |
| 04 资产管线对齐 | `04-asset-pipeline-alignment.md` | 可与 03 并行 | in_progress（M0 asset facade/reference gap table 完成并回写 `.zmeta` processor/meta owner 口径；M1.1 句柄语义裁决为保留 Zircon `Copy` typed ID，悬挂 handle 查询返回 `NotLoaded` 的命名回归测试已新增；M1.2 资源状态机收紧已落地，`Error -> Ready` 必须经 `Reloading`，失败原因通过 `ResourceRecord` diagnostics 投影到 facade；M2.1 worker pool 已切到 `AssetWorkerPoolOptions`，显式无界模式与 bounded queue-full 错误已落地；M2.2 pool-local in-flight request coalescing 已落地，`request_sender()` 暂留为低层旁路；M2.3 worker diagnostics 计数与 `DiagnosticStore` 发布已落地，in-flight 注册顺序已收紧以避免快完成悬挂计数；M3 watcher debounce/options、watch error 订阅面与 typed hot-reload event 回归已落地；watcher 复验第二次进入测试阶段，4 个低层 watcher 用例通过，3 个既有 manager watcher 用例被 Scene `.zasset` bincode 读回阻塞；asset cache wire type 已补 scene mesh/camera/collider/joint 场景缓存修复并拆入 `asset/artifact/cache_payload/scene.rs`，`cargo test -p zircon_runtime --lib artifact_store_roundtrips_scene_assets_with ...` 已通过 4/4，watcher acceptance 已通过 7/7） |
| 05 scene/editor 边界收尾 | `05-scene-editor-boundary-closeout.md` | 无（最小） | completed（M1/M2 完成；scoped Cargo 通过） |
| 06 插件公开面与生命周期 | `06-plugin-surface-and-lifecycle.md` | 02 | in_progress（M1 binding 空参数修复通过；runtime 真实后端验证待重跑） |
| 07 runtime 侧性能热路径 | `07-runtime-performance-hotpath.md` | 03;ZrVM assertion 解除;render 计划阶段 A | in_progress（M1.1 ECS QueryState 本地 cache telemetry 已落第一段：hits/misses/rebuilds/candidate/matched/revision 快照、行为断言与结构守卫完成，静态格式/行数检查通过；2026-06-13 聚焦 Cargo 未执行到新增用例，先被活跃 plugin bridge 切片的 `extension_registry_bridge.rs` unresolved `BridgeInterfaceSnapshot` / `BridgeInterfaceStatus` / `BridgeOwnerTransitionReport` 导入阻断） |
| 08 ECS 内核数据面对齐 | `08-ecs-kernel-data-alignment.md` | 与 03/07 同文件区错峰；08↔10 实体 ID ABI 互检 | in_progress（M0 数据面对照裁决完成；M1 生命周期测试矩阵四条测试已落地；M2.1 观察者同步触发/目标实体/dispatch 移除三条测试已落地；M2.2 命令队列错误报告面与两条测试已落地；M3.1 events/messages 双通道四条测试已落地；M3.2 change tick 回绕与 stale 窗口截断两条测试已落地；Cargo 待活动 lanes 清空后运行） |
| 09 UI 子系统架构收束 | `09-ui-subsystem-architecture.md` | 05 debt 移交清单；editor UI 会话稳定；文本栈归 01-M2 | planned（2026-06-12 新增） |
| 10 dynamic_api 与 interface 收敛 | `10-dynamic-api-and-interface-convergence.md` | 06（native ABI 同口径互引）；09-M0（镜像重复清单移交） | in_progress（M0 ABI 清册/版本策略完成；M1.1 ABI-safe/清册/版本策略守卫已扩展并通过完整 `zircon_runtime_interface` 包测试 165/165；M1.2 headless/minimal 跳过 render bridge 与 live destroy 夹具代码完成且静态检查通过，`destroy_session` Cargo 验证在活动编译通道下 904s 超时未声明通过；M3.1 loader 失败路径含真实缺符号 fixture 的 scoped app Cargo 已通过，完整 app 包待测，2026-06-13） |

阶段划分:

- 阶段 A（低风险、可立即）:05 + 01 的决策记录切片。纯删除/文档/守卫，不与活动会话冲突。
- 阶段 B（结构收束）:02 → 06；10 的 M0/M1（清册 + ABI 守卫，纯文档/守卫件）可与 06 并行，M2/M3 在 06-M2 收口后执行。
- 阶段 C（子系统对齐）:03 与 04 并行；08 的 M0/M1 可与 03 并行（同文件区切片错峰），M2/M3 在 03-M2 后执行；09 在 editor UI 会话稳定 + 05 收尾闭环后启动（M0 纯文档可提前）。
- 阶段 D（性能）:07。前置:ZrVM 空指针修复、render 计划 01/02 落地;08 的"债证明"锚定测试可作为 07-M1 计数的语义基线。

## 4. 全局边界约束（各子计划必须遵守）

继承 `Runtime 吸收层与 Editor_Scene 边界收束计划.md` 与 render 总计划 §5:

1. 不新增 crate;公共架构保持 `zircon_app`/`zircon_runtime`/`zircon_editor` 三件套 + `zircon_runtime_interface` ABI 层 + 内部 `core/{runtime,framework,manager,math,resource}` spine。
2. 硬切换:新 owner 路径落地的同一变更内迁移调用方并删除旧路径，不留 re-export、alias、shim。
3. 非网络语义的 `server` 命名是 blocker（`target-server` 为真实 headless 服务宿主语义，合法）。
4. 动态边界（dynamic_api、native 插件、VM 插件）只传 ABI-safe 值与序列化负载。
5. generated 产物只许 leaf binding/DTO/table，不许持有业务规则、调度或状态突变。
6. 渲染骨架（RDG、MeshDrawCommand、GPUScene、可见性、光照、时域、后处理、permutation）一律归 `docs/plans/zircon_runtime/render/01-08`，本目录子计划不得重复或冲突。

## 5. 全局验收与测试基线

按 milestone-first 政策:实现切片期间只做轻量检查，每个里程碑末进入测试阶段。

- 切片期:`cargo check -p zircon_runtime --lib --locked`（必要时 `-p zircon_app` / `-p zircon_editor`）。
- 里程碑测试阶段:`cargo test -p zircon_runtime --lib --locked`（按子计划模块过滤词收窄）;涉及装配时加 `cargo test -p zircon_app --locked`。
- 插件接缝:`cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --all-targets --locked`。
- 结构守卫:各子计划列出的源断言/结构测试（如 app 不得直依插件实现 crate、interface 不得出现 wgpu）。
- 文档:每个里程碑完成后按源码镜像路径更新 `docs/zircon_runtime/**`，并刷新本目录子计划状态标记。

## 6. 协调与活动会话避让

- `20260604-1232-runtime-architecture-review`（活跃）:正在执行旧渐进式计划的切片（root surface 债 4、大文件债 5、hotspots 37 的审计口径以该会话为准）。本目录 02/05 子计划的执行必须先对齐该会话的最新 touched_modules，避免双写。
- `20260611-0416-rendering-10fps-analysis`（活跃）:graphics 性能修复进行中，明示"不回退 worktree 改动、只做聚焦编辑"。07 子计划执行前必须复读该笔记。
- `20260603-2304-plugin-ecosystem-continuation`（活跃）:06 子计划执行前同上。

## 7. 工程化落地公约（2026-06-12 细化定稿，约束全部子计划执行）

各子计划已细化到切片级；执行任何切片时遵守以下公约，违反视为切片未完成：

1. **切片五要素**:每个切片的"目标文件 / 改动形态 / 调用方迁移 / 验收 / DoD"五项缺一即不开工;签名草案在动手前定稿，定稿差异回写计划文件。
2. **执行前检查清单是闸门**:各子计划"执行前检查清单"逐项过完才能动第一刀;行号与计数以重核结果为准，漂移时先回写"现状与证据"节再继续。
3. **状态节实时性**:每完成一个切片，立刻更新该计划"状态与产出记录"表（状态/日期/证据三列），禁止批量补记;基线数值在开工首日填写。
4. **milestone-first 测试节奏**:切片期只跑该计划列出的 `cargo check`;测试统一压到里程碑末的"测试阶段"命令清单，逐条可复制执行并留存输出摘要。
5. **硬切换提交粒度**:`git mv` / 删导出 / 改签名类切片，旧路径删除与调用方迁移必须同一提交闭合，禁止中间态（双签名、临时 re-export）跨提交存在。
6. **会话避让**:触及 `20260604-1232`（架构审查）、`20260611-0416`（10fps，禁止回退其改动）、`20260603-2304`（插件生态）三个活跃会话工作区前，先重读其笔记并按计划"风险与协调"节对齐;每次执行开新会话笔记（cross-session-coordination 规范）。
7. **证据纪律**:计划中标注"执行时核验:<命令>"的条目，核验输出粘入状态节;新增守卫测试一律带负例自检（参照 `authoring_boundary_guard_fails_on_representative_tokens` 模式）。
8. **共享基建复用**:结构扫描守卫（02-M4 generated、05-1.4 命名、01-1.4 manifest）共享同一套源码遍历 helper，后落地者复用先落地者，禁止三套扫描实现。
