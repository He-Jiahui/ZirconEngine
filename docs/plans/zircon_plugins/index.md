# Zircon 插件生态完善总体计划（docs/plans/zircon_plugins）

> 状态：工程化细化版 v2（2026-06）——全部分计划已按代码实查校准基线，并细化到"模块文件树 + API 签名 + 里程碑任务表（含测试名）"的可开工粒度
> 范围：以插件架构核心升级为前置，系统性补齐 Sound / Physics / Animation / Navigation / AI / Net / VM / 跨平台发行 八大周边设施插件，并确立编辑器集成横切规范。
> 原则（全计划共同遵守）：硬切不留兼容层；中立契约在 `zircon_runtime::core::framework::*`，实现在 `zircon_plugins/*`；root 接线文件保持薄；注册期重、运行期零开销；插件分 runtime/editor 两部分注册。
> **实现纪律**：每份计划文末附录给出 `dev/` 参考源码对位表（路径均已实地核验）——任务开工前必读对应条目，复杂算法/协议/语义（调度拓扑、DSP 数值、Jolt 映射、行为树打断、复制协议、导出流程等）一律对照参考实现核对，**禁止凭记忆或凭空实现**。

## 1. 文档索引

| 编号 | 计划 | 优先级 | 核心交付 |
|------|------|--------|---------|
| [01](01-plugin-architecture-core.md) | 插件架构核心升级 | **P0** | 9 阶段调度 + SystemSet 偏序、register_system/resource/event、TypedExtensionPoint/FrozenExtensionTable、四阶段生命周期、Editor capability 对称、Native ABI v3 |
| [02](02-sound.md) | Sound | P1 | **kira 为执行核心**（自研混音/输出栈退役）、契约 → kira track/effect 映射编译、缺口效果与 HRTF 以 kira Effect 迁移现有数值、3D 策略计算 + Timeline→Tween/Clock |
| [03](03-physics.md) | Physics | P1 | PhysicsBackend trait + joltc-sys 后端、形状全集、六约束、ragdoll 三模式、QueryMode/过滤 |
| [04](04-animation.md) | Animation | P1 | AnimationTargetId dense 化 + PoseBuffer SoA 零分配求值、avatar mask 编译、多层状态机、GPU skinning、IK |
| [05](05-navigation.md) | Navigation | P1 | TiledBake 异步、DetourCrowd 接入、TileCache carving、off-mesh traverse 状态机（vendored recastnavigation 已在） |
| [06](06-ai.md) | AI | P2 | 行为树 dense 编译 + 执行内核重写（observer aborts）、类型化 blackboard、perception 预算、集成节点 |
| [07](07-net.md) | Net | P2 | NetWorker 线程模型纠偏（替代 block_on）、TLS、Session/RPC、replication 闭环、可靠 UDP、内容下载 |
| [08](08-zr-vm.md) | ZrVM | P2 | zircon_reflect_derive 宏、双向反射统一模型、dense call site、四回调通道、GC 协约、热替换迁移 |
| [09](09-export-publishing.md) | 跨平台发行 | P2 | 导出 profile、三路径闭环、平台模板、zrpack 字节格式与裁剪、`zircon export` CLI 阶段状态机 |
| [10](10-editor-integration.md) | 编辑器集成规范 | P1（横切） | 扩展点签名级约定、AI Workbench 对位表、三项调试设施、E1 反射默认 drawer、E2 OperationStack 收口 |
| [11](11-plugin-call-bridge.md) | 插件调用桥 | P1（横切） | 强/弱依赖接口直调（StrongBridge 零检查 / WeakBridge 未启用返回 NotEnabled）、FrozenBridgeTable 世代模型、事件 dense 通道 + dormant 订阅 |
| [12](12-plugin-dx-and-structure-framework.md) | 插件 DX 与结构框架 | P1（横切） | 统一 `plugin.toml` schema + 唯一 crate 骨架 + 注册单入口 + capability 单源（四源一致）+ `plugin_sdk` builder；执行 [`engine-code-structure-convention`](../engine-code-structure-convention.md) §6 |
| [13](13-standalone-plugin-build.md) | 插件独立构建与分发 | P1（横切） | 双形态单源（embed `rlib` / dist `cdylib`）+ 依赖边界硬约束（dist 闭包禁 `zircon_runtime`）+ 注册跨 ABI 编组 + `zircon plugin build <id>` 每插件独立构建 + per-plugin zrpack 资产子包 + ABI/引擎兼容性协商；规范见 [`plugin-standalone-build.md`](../../zircon_plugins/plugin-standalone-build.md) |

## 2. 依赖图与推进顺序

```
01 插件架构核心 (P0)
 ├─ M1 调度/系统注册 ──→ 03-M1 / 04-M1（首批验证者，同窗口迁移 scene hook）
 ├─ M2 资源/事件/类型化扩展点 ──→ 02 / 05 / 06 / 07 全部
 ├─ M3 生命周期/可选功能 ──→ 07（feature 包）、可选依赖探测（02 occlusion、05 几何收集、06 sight）
 ├─ M4 Editor 对称化 ──→ 10-E2、各插件 Editor 里程碑
 └─ M5 Native ABI v3 ──→ 09-M5（NativeDynamic 发行）

08-M1 统一反射（zircon_reflect_derive） ──→ 10-E1（默认 drawer）、07-M3/M4（RPC/replication schema）、06-M3（ScriptTask）
01-M2/M3 ──→ 11-M1（调用桥核心）；11-M1 ──→ 02 occlusion / 05 几何收集 / 06 sight 的运行期弱依赖调用（WeakBridge<physics.query.v1>，physics 于 03-M2 后导出）
11-M2 事件优化 ──→ 与 01-M2 register_event 同窗口落地（EventStore dense 化一次迁移）
04-M2 骨骼姿态通道（SkeletalPoseTargets/SimulatedPoseFeed） ──→ 03-M5（ragdoll）
05-M3 agent ──→ 06-M3（MoveTo 节点）
07-M6 content_download ↔ 09-M2 zrpack（共享 ZrPackManifest/ZrChunkEntry DTO，定义于 framework/net/download.rs，先定 DTO 再各自实现）
13 插件独立构建（双形态/依赖边界/ABI 编组/per-plugin build）依赖 01-M2/M4（register 通道+访问集）、09-M1/M2（NativeDynamic+zrpack 底座）、11-M1/M2（bridge dense 通道）、12-M1..M4（manifest/骨架/SDK/capability）；其 M1/M2 与 12 同窗口（双形态骨架是 12 骨架的发行维扩展），M3/M4 与 09-M1/M2 共享 zrpack/CLI 底座，M5 随各能力波次 touch-it-conform-it
```

推荐执行波次（同波内可并行，互不依赖）：

1. **波次零（结构前置）**：12-M1/M2/M3/M4（统一 `plugin.toml` schema、唯一 crate 骨架 + `plugin_sdk`、注册单入口、capability 单源）+ **13-M1/M2（双形态骨架 + 依赖边界 + 注册跨 ABI 编组）**——先于能力波次，定义所有插件共用契约；12-M5 / 13-M5 存量硬切随各插件能力波次 touch-it-conform-it（见计划 12 / 13）。2026-06-22 至 2026-06-23 已推进 12-M1 manifest/schema/generated parity、12-M2 `plugin_sdk` builder/native/editor/test fixture 骨架，以及 12-M3 registration builder + animation 代表迁移、`asset_importers/{data,model,shader}` family 与 split importers（`gltf_importer` / `obj_importer` / `texture_importer` / `audio_importer` / `opus_importer` / `shader_wgsl_importer` / `ui_document_importer`）公开注册自由函数清零，并新增 D12 `runtime_plugin_exports!` SDK helper，完成 ai、animation、hybrid_gi、navigation、net、particles、physics、prefab_tools、rendering、solari、terrain、texture、tilemap_2d、virtual_geometry、zr_vm_language 共 15 个 trait-backed runtime 插件的 helper 全量 rollout。当前状态为 `plugins_12_runtime_capability_single_source_guard_passed`：上述 15 个 trait-backed first-party runtime 根的 runtime package capability 已迁入 `runtime/src/capability.rs`，`tools/audit_plugin_structure.py --json` 报告 `missing_plugin_toml = 0`、`manifest_schema_violations = 0`、`m1_gate_status = classified-and-clear`、`skeleton_conformance.sample_conformance_status = sample-clean`、`sample_workspace_dependency_status = sample-workspace-deps-clean`、`migration_debt_count = 35`、`registration_conformance.m3_t1_gate_status = family-single-entry-clean`、`registration_conformance.m3_split_importer_gate_status = split-importer-single-entry-clean`、`asset_importer_family_free_function_registration_sites = 0`、`split_importer_free_function_registration_sites = 0`、aggregate `m3_importer_gate_status = importer-single-entry-clean`、`capability_conformance.audited_runtime_root_count = 15`、`capability_conformance.capability_source_mismatches = 0`、`capability_conformance.m4_runtime_capability_gate_status = runtime-capability-single-source-clean`。native-only SDK check/test、native fixture check、SDK editor feature check、editor sample check、default SDK test-runtime check、`test_runtime_builder` SDK self-test 2/2、registration builder SDK self-test 1/1、animation registration focused test 1/1、SDK+animation locked check、data/model/shader importer family locked check、split importer focused `cargo check --locked`、D12 full rollout rustfmt、16-package D12 offline/locked check、M4 capability py_compile/audit JSON/scoped rustfmt、16-package M4 focused `cargo check --locked` 与 catalog `plugins_12_capability_single_source_conformance` focused test 1/1 已通过（仅既有 runtime/large-plugin warnings）。editor feature/sample focused Cargo tests 仍为编译期超时、不计通过；model focused lib-test 被既有 `MaterialCaptureSeed` / `MaterialRuntime::capture_seed` runtime lib-test drift 阻断；`asset_importers/audio` 与 `asset_importers/texture` 的 legacy `package_manifest()` 仍归 importer owner 收口，不属于 D12 trait-backed helper 块；M4/T2 editor `mirrors_runtime(...)`、optional feature capability builder、sound/importer/editor 更广迁移与 M5 存量迁移仍未关闭。
1. **波次一**：01-M1/M2 + 03-M1 + 04-M1（架构核心与两个首批验证插件）
2. **波次二**：01-M3/M4 + 11-M1/M2 + 02-M1/M2 + 05-M1/M2 + 08-M1
3. **波次三**：03-M2..M4 + 04-M2..M4 + 05-M3..M5 + 06-M1/M2 + 07-M1..M3 + 11-M3
4. **波次四**：03-M5 + 04-M5 + 06-M3/M4 + 07-M4..M6 + 08-M2..M4 + 09-M1/M2 + **13-M3/M4（每插件独立构建命令 + 产物包/兼容性协商，与 09 共享 zrpack/CLI 底座）** + 01-M5 + 11-M4
5. **波次五**：全部 Editor 里程碑（按 10 规范，含 10-E1/E2）+ 09-M3..M6 + 08-M5 + **13-M5（双形态全量 rollout + 每插件 CI 构建矩阵）**

## 3. 现状基线摘要（2026-06 代码实查结论，v2 校准）

- **架构**：四层插件系统（package_manifest / runtime_plugin / extension_registry / native_plugin_loader）成型；**访问集（`SystemParamAccess`）、冲突图、并行批次、7 阶段 `SystemStage` 已存在**——真缺口是 SystemSet 偏序、registry 的 system/resource/event 注册通道、类型化扩展点冻结、owner 追踪、ABI 分域函数表（01 §2 缺口表 G1–G8）。
- **插件成熟度（实查比早期调研乐观）**：sound 自研执行栈功能面完整但热路径有硬伤，**已裁决以 kira 为执行核心整体替代自研混音/输出栈（02 v2.1），自研 DSP 数值迁移为 kira 自定义 Effect**；navigation 已 vendored upstream recastnavigation 且单块烘焙 bridge 在（缺 tiled/crowd/carving 闭环）；net 六 feature crate 已建（缺 worker 线程模型与深度实现）；physics/animation 契约 DTO 完整、核心算法缺（Jolt、约束、ragdoll、GPU skinning、avatar mask 执行）；ai 框架在、节点库/观察者中断/感知缺；zr_vm 为 wrapper 但**反射 DTO 家族与宿主注册表已在**（缺 derive/反向/性能层）；export 的 profile/验证/物化模板**已在**（缺 zrpack/CLI/平台模板）；terrain/tilemap_2d 为 stub（不在本轮范围）。
- **既有计划冲突已裁决**：Physics 后端以"Jolt（joltc-sys）唯一必交付 + builtin 降级"为准（03）；animation 归属以"已迁入 zircon_plugins"现实为准（04）；Tokio 不进 runtime 本体依赖、由 net 插件自建 worker（07）。

## 4. 全局验收

每波次收口必须全绿：

```bash
cargo build --workspace --locked
cargo test --workspace --locked
cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --all-targets --locked
cargo test --manifest-path zircon_plugins/Cargo.toml --workspace --locked
cargo fmt --all --check
```

并保持四源一致性测试（plugin.toml / runtime descriptor / 派生 builtin catalog / workspace member）覆盖每个新增 feature 与模块；01-M4-T3 起新增 `system_anchors` 声明与实际注册的契约核对。

## 5. 与既有计划的关系

本目录是对 `.codex/plans/` 中相关计划（周边设施、Sound 核心、Physics+Animation、Net、导航、ZrVM、全量插件化收敛、多插件组合可选功能、插件注册与 EditorOperation、最小本体与发行导出）的**收敛与细化**：既定架构决策全部继承，冲突项在各分计划中显式裁决并标注；后续以本目录为插件生态的执行基线，`.codex/plans` 对应文档转为历史背景材料。

v2 较 v1 的实质变化：各计划的"现状基线"按代码实查重写（多处早期调研结论被修正，见各文档 §2 缺口表的证据列）；新增模块文件树、Rust 签名级 API、里程碑任务表（任务 id/改动文件/依赖/测试函数名）；跨文档接口（03↔04 ragdoll、05↔06 MoveTo、07↔09 zrpack、08↔07 schema、08↔10 drawer）双侧签名对齐。
