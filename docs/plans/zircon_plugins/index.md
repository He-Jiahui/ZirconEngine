# Zircon 插件生态完善总体计划（docs/plans/zircon_plugins）

> 状态：工程化细化版 v2（2026-06）——全部分计划已按代码实查校准基线，并细化到"模块文件树 + API 签名 + 里程碑任务表（含测试名）"的可开工粒度
> 范围：以插件架构核心升级为前置，系统性补齐 Sound / Physics / Animation / Navigation / AI / Net / VM / 跨平台发行 八大周边设施插件，并确立编辑器集成横切规范。
> 原则（全计划共同遵守）：硬切不留兼容层；中立契约在 `zircon_runtime::core::framework::*`，实现在 `zircon_plugins/*`；root 接线文件保持薄；注册期重、运行期零开销；插件分 runtime/editor 两部分注册。

## 1. 文档索引

| 编号 | 计划 | 优先级 | 核心交付 |
|------|------|--------|---------|
| [01](01-plugin-architecture-core.md) | 插件架构核心升级 | **P0** | 9 阶段调度 + SystemSet 偏序、register_system/resource/event、TypedExtensionPoint/FrozenExtensionTable、四阶段生命周期、Editor capability 对称、Native ABI v3 |
| [02](02-sound.md) | Sound | P1 | CompiledMixGraph 零分配热路径重构、DspEffect 统一 trait + ParametricEq、SPSC 命令队列线程模型、声道协商、Timeline 自动化 |
| [03](03-physics.md) | Physics | P1 | PhysicsBackend trait + joltc-sys 后端、形状全集、六约束、ragdoll 三模式、QueryMode/过滤 |
| [04](04-animation.md) | Animation | P1 | AnimationTargetId dense 化 + PoseBuffer SoA 零分配求值、avatar mask 编译、多层状态机、GPU skinning、IK |
| [05](05-navigation.md) | Navigation | P1 | TiledBake 异步、DetourCrowd 接入、TileCache carving、off-mesh traverse 状态机（vendored recastnavigation 已在） |
| [06](06-ai.md) | AI | P2 | 行为树 dense 编译 + 执行内核重写（observer aborts）、类型化 blackboard、perception 预算、集成节点 |
| [07](07-net.md) | Net | P2 | NetWorker 线程模型纠偏（替代 block_on）、TLS、Session/RPC、replication 闭环、可靠 UDP、内容下载 |
| [08](08-zr-vm.md) | ZrVM | P2 | zircon_reflect_derive 宏、双向反射统一模型、dense call site、四回调通道、GC 协约、热替换迁移 |
| [09](09-export-publishing.md) | 跨平台发行 | P2 | 导出 profile、三路径闭环、平台模板、zrpack 字节格式与裁剪、`zircon export` CLI 阶段状态机 |
| [10](10-editor-integration.md) | 编辑器集成规范 | P1（横切） | 扩展点签名级约定、AI Workbench 对位表、三项调试设施、E1 反射默认 drawer、E2 OperationStack 收口 |
| [11](11-plugin-call-bridge.md) | 插件调用桥 | P1（横切） | 强/弱依赖接口直调（StrongBridge 零检查 / WeakBridge 未启用返回 NotEnabled）、FrozenBridgeTable 世代模型、事件 dense 通道 + dormant 订阅 |

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
```

推荐执行波次（同波内可并行，互不依赖）：

1. **波次一**：01-M1/M2 + 03-M1 + 04-M1（架构核心与两个首批验证插件）
2. **波次二**：01-M3/M4 + 11-M1/M2 + 02-M1/M2 + 05-M1/M2 + 08-M1
3. **波次三**：03-M2..M4 + 04-M2..M4 + 05-M3..M5 + 06-M1/M2 + 07-M1..M3 + 11-M3
4. **波次四**：03-M5 + 04-M5 + 06-M3/M4 + 07-M4..M6 + 08-M2..M4 + 09-M1/M2 + 01-M5 + 11-M4
5. **波次五**：全部 Editor 里程碑（按 10 规范，含 10-E1/E2）+ 09-M3..M6 + 08-M5

## 3. 现状基线摘要（2026-06 代码实查结论，v2 校准）

- **架构**：四层插件系统（package_manifest / runtime_plugin / extension_registry / native_plugin_loader）成型；**访问集（`SystemParamAccess`）、冲突图、并行批次、7 阶段 `SystemStage` 已存在**——真缺口是 SystemSet 偏序、registry 的 system/resource/event 注册通道、类型化扩展点冻结、owner 追踪、ABI 分域函数表（01 §2 缺口表 G1–G8）。
- **插件成熟度（实查比早期调研乐观）**：sound 执行引擎/DSP/HRTF/occlusion/多声道**已实现**但热路径违反零分配原则（02 S1/S2）；navigation 已 vendored upstream recastnavigation 且单块烘焙 bridge 在（缺 tiled/crowd/carving 闭环）；net 六 feature crate 已建（缺 worker 线程模型与深度实现）；physics/animation 契约 DTO 完整、核心算法缺（Jolt、约束、ragdoll、GPU skinning、avatar mask 执行）；ai 框架在、节点库/观察者中断/感知缺；zr_vm 为 wrapper 但**反射 DTO 家族与宿主注册表已在**（缺 derive/反向/性能层）；export 的 profile/验证/物化模板**已在**（缺 zrpack/CLI/平台模板）；terrain/tilemap_2d 为 stub（不在本轮范围）。
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
