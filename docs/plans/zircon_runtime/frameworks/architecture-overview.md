---
related_code:
  - Cargo.toml
  - zircon_app/src/plugins/groups.rs
  - zircon_app/src/entry/export_bootstrap.rs
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/core/mod.rs
  - zircon_runtime/src/core/runtime/mod.rs
  - zircon_runtime/src/builtin/runtime_modules/core_modules.rs
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_runtime/src/plugin/native_plugin_loader/mod.rs
  - zircon_runtime_interface/src/lib.rs
  - zircon_editor/src/lib.rs
  - zircon_plugins/Cargo.toml
plan_sources:
  - docs/plans/zircon_runtime/frameworks/index.md
  - docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
  - docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md
---

# Zircon 引擎目标架构示意图

本文是 frameworks 计划集的配套图集：用固定的文字示意图描述**目标态**架构（含计划 01–05 落地后的形态）。与现状不一致处均为计划内容，勾稽对应子计划编号。图中所有层次与箭头方向是规范性的（normative）：新增代码若与图冲突，以图与 `development-conventions.md` 为准，或先修订本图再动代码。

## 1. 总体形态：进程与包

```
┌────────────────┐  启动子进程   ┌───────────────────────────────────────────┐
│  zircon_hub    │ ───────────▶ │  zircon_app                                │
│  桌面启动器/安装 │              │  进程入口 · profile 选择 · 插件组合          │
│  (唯一 Slint 户)│              │  窗口/主循环宿主 · libloading 装载 runtime   │
└────────────────┘              └────────┬─────────────────────┬────────────┘
                                         │ target-editor-host  │ target-client/server
                                         │ (静态链接 editor)     │
                              ┌──────────▼─────────┐           │
                              │  zircon_editor     │           │
                              │  作者态宿主          │           │
                              │  core/ scene/ ui/  │           │
                              └──────────┬─────────┘           │
                                         │ 运行时客户端/会话      │
                                         ▼                     ▼
                              ┌─────────────────────────────────────────────┐
                              │  zircon_runtime（rlib + cdylib）             │
                              │  引擎实现：门面 + 内部 zr_* 分层 crate（图 2）  │
                              │  dynamic_api 会话出口（ZrRuntimeApiV1）       │
                              └──────────┬──────────────────▲───────────────┘
                                         │ 消费契约/DTO/句柄   │ ABI v3 函数表
                              ┌──────────▼──────────────────┴───────────────┐
                              │  zircon_runtime_interface（稳定 ABI 层）      │
                              │  handles · status · 函数表 · manifest DTO ·  │
                              │  reflect/resource 契约 · ui 中立契约          │
                              └──────────────────▲──────────────────────────┘
                                                 │ 唯一公共依赖
                              ┌──────────────────┴──────────────────────────┐
                              │  zircon_plugins（独立 workspace）             │
                              │  first-party 插件族 + plugin_sdk + catalog   │
                              │  linked（rlib，feature 选链）│ dist（cdylib）  │
                              └─────────────────────────────────────────────┘
```

要点：

- 动态边界（app↔runtime、runtime↔native 插件）只传 ABI 安全值与序列化载荷；Rust trait 对象、wgpu/Slint 对象、world 引用不过界。
- `zircon_editor` 不拥有运行时世界；它经 runtime 客户端与契约消费世界快照，作者态（选择集/viewport 工具/gizmo）只存在于 editor（收束计划边界，不变）。
- `zircon_hub` 是唯一的 Slint 依赖户，永不回流 editor。

## 2. `zircon_runtime` 内部 crate 分层（计划 01 目标态）

依赖方向严格自下而上；同层横向依赖必须经 `zr_contracts` 或在计划 01 §3 显式批准。

```
facade   ┌─────────────────────────────────────────────────────────────────┐
         │ zircon_runtime 门面：builtin 模块组装 · plugin 加载 · dynamic_api │
         │ core/manager resolver 组装 · prelude · curated re-export · cdylib│
         └───────▲──────────▲──────────▲──────────▲──────────▲─────────────┘
optional │ zr_script │ zr_animation │ zr_navigation │        （feature 可选） │
layer 5  ┌─────────────────┐                                                │
         │      zr_ui      │  布局/模板/组件/表面                             │
layer 4  ├─────────────────┼─────────────────┐                              │
         │   zr_graphics   │     zr_text     │  渲染内核 · 共享文本服务        │
layer 3  ├─────────┬───────┴───┬─────────────┤                              │
         │ zr_rhi  │ zr_rhi_wgpu│zr_render_graph│  RHI 契约/wgpu 后端/渲染图   │
layer 2  ├─────────┴───┬───────┴─────────────┤                              │
         │  zr_asset   │      zr_scene       │  资产管线 · ECS 世界           │
layer 1  ├─────────────┼──────────┬──────────┤                              │
         │ zr_platform │ zr_input │zr_diagnostics│  平台(winit 收拢)/输入/诊断 │
layer 0  ├─────────────┴──────────┴──────────┴──────────────────────────────┤
         │  zr_kernel（生命周期/调度/描述符） · zr_contracts（纯契约,按域门控）  │
         │  zr_math（转发 interface::math） · zr_resource（句柄/注册表/租约）   │
         └─────────────────────────────────────────────────────────────────┘
```

铁律：layer 0 禁止 wgpu/winit 等重依赖；wgpu 只允许出现在 `zr_rhi_wgpu`/`zr_graphics` 依赖树；`zircon_app`/`zircon_editor`/插件永远只依赖门面，直连 `zr_*` 是架构违规（守卫 G1）。

## 3. 启动与模块生命周期（计划 02 目标态）

```
zircon_app 入口
  │ 解析 EntryConfig → RuntimeProfileId → feature 预设校验（计划 03 单源表）
  │ 收集描述符：builtin 模块 + linked 插件(catalog) + native 插件(manifest 扫描)
  ▼
CoreRuntime 注册全部 ModuleDescriptor           状态: Registered
  ▼
按 InitLevel 逐层推进:
Kernel ──▶ Services ──▶ Scene ──▶ Editor* ──▶ Post     (*仅 editor-host)
  每层内: 依赖拓扑排序
    ├─ build(ctx)   注册服务/系统/事件            状态: Initializing
    ├─ ready(ctx)?  异步就绪轮询(GPU surface 等,带超时预算)
    └─ finish(ctx)  全员 ready 后收尾接线
  ▼
Active: 帧循环（图 4）                            状态: Running
  ▼
关停: cleanup(ctx) 按反向依赖序                    状态: Stopping → Unloaded
  规则: Driver 晚于依赖它的 Manager/System/Plugin 卸载
```

插件默认挂 `InitLevel::Post`；声明更早层级需给出能力依据（规范 PL-7）。热重载走同一状态机：`save_state → cleanup(旧) → build/ready/finish(新) → restore_state`，宿主句柄稳定、实例可替换。

## 4. 帧循环与渲染数据流

```
主循环 tick (zircon_app 宿主驱动)
  ┌────────────────────────── 模拟侧 (world 权威在 zr_scene) ──────────────────┐
  │  PreUpdate → Update → LateUpdate → FixedUpdate(积累步) → RenderExtract      │
  └──────────────────────────────────────────────┬───────────────────────────┘
                                    extract packet / snapshot DTO
                                    (zr_contracts::render, 只读, 无 world 引用)
                                                  ▼
  ┌────────────────────────────── 渲染侧 (zr_graphics) ────────────────────────┐
  │  render graph 编排 → prepare/queue → RHI 提交 (zr_rhi_wgpu) → present      │
  └────────────────────────────────────────────────────────────────────────────┘
  editor-host 变体: editor 经会话取 viewport 快照,叠加 overlay packet(作者态)后合成
```

渲染语义（pass 结构、GPUScene、时域链等）以 `docs/plans/zircon_runtime/render/` 为权威；本图只锁定"模拟与渲染之间只有 DTO"这一组织约束。

## 5. 插件系统双路径与元数据单源（计划 04 目标态）

```
                 declare_plugin!{ … }  ← 插件唯一元数据源 (runtime crate)
                        │ 宏展开
        ┌───────────────┼─────────────────────────────┐
        ▼               ▼                             ▼
  常量/描述符/入口   plugin.toml（生成物,CI 校验一致）   dist 符号(\0 字节串等)
        │
  ┌─────┴──────────────────────────┬────────────────────────────────────┐
  │ 路径 A: linked（第一方默认）      │ 路径 B: native_dynamic（导出/三方）   │
  │ first_party_runtime_catalog     │ export_root/plugins/*.dll|so 扫描   │
  │ feature 选链 → 注册函数           │ libloading → descriptor_v3 符号     │
  └─────┬──────────────────────────┴──────┬─────────────────────────────┘
        ▼                                 ▼ ABI 版本校验·manifest 解析·能力协商
   RuntimePluginDescriptor（内嵌 ModuleDescriptor, 计划 02）
        ▼
   统一注册: RuntimeExtensionRegistry（扩展槽）+ InitLevel 排序 → 生命周期图 3
   失败路径: PluginLoadError 类型树（阶段·期望/实际·修复提示）→ 结构化报告
```

## 6. Editor ↔ Runtime 权威边界（收束计划,不变）

```
┌─权威──────────────────────────────┐        ┌─权威──────────────────────────┐
│ zircon_editor                     │        │ zircon_runtime (zr_scene)      │
│ · 选择集/viewport 工具/相机 override│        │ · entity/hierarchy/transform   │
│ · gizmo/handle/overlay 生成        │        │ · 组件/序列化(不含 selection)    │
│ · 命令/undo/意图/事件 journal       │        │ · 调度/RenderExtract           │
└──────────────┬────────────────────┘        └───────────▲───────────────────┘
               │  编辑命令(意图 DTO) ────────────────────────┘
               │  世界快照/层级查询投影 ◀───────────────────── 
               └── overlay packet(中立 DTO, zr_contracts::render) → 渲染合成
```

编辑器场景树只是 ECS 层级的查询投影；world 序列化不含任何作者态。

## 7. 守卫映射速查

| 图中约束 | 守卫 |
|---------|------|
| 分层依赖方向 / 禁直连 zr_* | G1（Cargo + 元数据守卫测试） |
| 动态边界只传 ABI 安全值 | interface 契约测试 + 评审（IF 规范） |
| 可选域可整体关断 | G4 feature 矩阵 CI |
| 插件元数据单源 | G5 `cargo zircon plugin check` |
| wgpu/winit 不越层 | G6 依赖树断言 |
