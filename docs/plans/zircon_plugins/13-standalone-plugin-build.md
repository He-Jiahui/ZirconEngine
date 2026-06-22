---
related_code:
  - zircon_plugins/Cargo.toml
  - zircon_plugins/Cargo.lock
  - zircon_plugins/animation/runtime/Cargo.toml
  - zircon_plugins/animation/runtime/src/lib.rs
  - zircon_plugins/animation/runtime/src/plugin.rs
  - zircon_plugins/physics/runtime/Cargo.toml
  - zircon_plugins/native_dynamic_fixture/plugin.toml
  - zircon_plugins/native_dynamic_fixture/native/Cargo.toml
  - zircon_plugins/native_dynamic_fixture/native/src/lib.rs
  - zircon_plugins/plugin_sdk/Cargo.toml
  - zircon_plugins/plugin_sdk/src/lib.rs
  - zircon_plugins/plugin_sdk/src/native.rs
  - zircon_plugins/plugin_sdk/src/registration.rs
  - zircon_plugins/plugin_sdk/src/runtime_exports.rs
  - zircon_plugins/plugin_sdk/src/manifest/package_builder.rs
  - zircon_runtime_interface/src/plugin_api.rs
  - zircon_runtime_interface/src/buffer.rs
  - zircon_runtime_interface/src/status.rs
  - zircon_runtime/src/plugin/export_build_plan/native_dynamic_package_plan.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_load_manifest.rs
  - zircon_runtime/src/builtin/runtime_modules/plugin_modules/loader.rs
  - tools/zircon_build.py
  - tools/zircon_export/cli.py
  - tools/zircon_export/native_build.py
  - tools/zircon_export/native_signing.py
  - tools/zircon_export/native_dynamic.py
  - tools/zircon_export/pipeline_report_native_dynamic_loader_manifest.py
  - tools/audit_plugin_structure.py
  - tools/plugin_structure_audits/manifest_schema.py
  - tools/plugin_structure_audits/capability.py
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
  - docs/plans/zircon_plugins/09-export-publishing.md
  - docs/plans/zircon_plugins/11-plugin-call-bridge.md
  - docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md
implementation_files:
  - docs/zircon_plugins/plugin-standalone-build.md
  - docs/zircon_plugins/plugin-manifest-schema.md
  - docs/zircon_plugins/plugin-crate-skeleton.md
  - docs/zircon_plugins/plugin-sdk.md
  - zircon_plugins/plugin_sdk/src/native.rs
  - zircon_plugins/plugin_sdk/src/registration.rs
  - tools/zircon_export/cli.py
  - tools/zircon_export/native_build.py
  - tools/zircon_build.py
  - tools/plugin_structure_audits/dependency_boundary.py
tests:
  - cargo metadata --manifest-path zircon_plugins/Cargo.toml --format-version 1 --no-deps --locked
  - python tools/audit_plugin_structure.py --json
  - cargo build --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_native_dynamic_fixture_native --locked
doc_type: structure-plan
status: planned
---

# 13 · 插件独立构建与分发计划（每插件可独立编译产出动态包）

> 状态：planned · 优先级：P1（横切，与 [09 跨平台发行](09-export-publishing.md)、[12 插件 DX 与结构框架](12-plugin-dx-and-structure-framework.md)、[11 调用桥](11-plugin-call-bridge.md) 并列横切层）
> 上游权威：[`engine-code-structure-convention`](../engine-code-structure-convention.md) §6（Plugin DX）、[Plugins 01](01-plugin-architecture-core.md)（`RuntimePlugin::register`、ABI v3、capability 协商）
> 规范落地（唯一权威）：[`docs/zircon_plugins/plugin-standalone-build.md`](../../zircon_plugins/plugin-standalone-build.md)
> 目标：把"每个插件都能脱离引擎源码树、仅依赖稳定 ABI 独立编译，产出可分发动态包并被 NativeDynamic loader 加载/热更"从 `native_dynamic_fixture` 一个 fixture，升格为所有一方插件都走的一等公民路径；静态链接（LibraryEmbed）降为发行期性能优化，而非唯一可构建形态。

## 1. 目标

1. **每插件独立构建**：`zircon plugin build <id>` 单命令、独立 target、独立产物目录，不依赖整 workspace 全量编译，不依赖 `path = ../../../zircon_runtime`。
2. **依赖边界硬约束**：插件的可分发形态只链接稳定 ABI（`zircon_plugin_sdk` 的 `native`/`dist` feature → `zircon_runtime_interface`），编译期 guard 断言其依赖闭包不含 `zircon_runtime`。
3. **双形态单源**：同一份插件声明投影出两种产物——in-tree `rlib`（LibraryEmbed 静态链接，零 FFI、最快）与 dist `cdylib`（NativeDynamic，ABI-only、可分发可热更）；二者由一份 manifest + 一份 backend 逻辑驱动，不复制代码。
4. **产物包规范**：`plugins/<id>/` { `<id>.{dll,so,dylib}` + `plugin.toml` + `native_dynamic_package.toml` 报告 + 可选 `<id>.zrpack` 资产子包 + 签名/hash }，被 `native_plugin_load_manifest` 收集、被 loader 加载。
5. **可重复构建与兼容性钉**：同输入 byte 相同产物；manifest 钉 `sdk_api_version` / ABI 版本 / 引擎兼容区间，loader 加载期协商，不匹配出结构化诊断而非崩溃。

## 2. 现状基线（实查，带路径证据）

独立构建的**工具链与 ABI 大半已在**，缺的是"对所有插件成立"和"依赖边界解耦"：

- **ABI v3 表面完整**：`zircon_plugins/plugin_sdk/src/native.rs` 已有 `NativePluginAbiV3`/`NativePluginEntryReportV3`/`NativePluginBehaviorV3`/`NativePluginHostFunctionTableV3`/`NativePluginBridgeMethodTableV3` 全套 `repr(C)`、SDK-owned byte buffer、panic guard、capability 协商（`host_supports_all_capabilities_v3`），以及 `native_command_plugin_v3!` / `export_native_plugin_descriptor_v3!` / `export_native_plugin_entry_v3!` helper 宏。
- **打包/加载契约在**：`export_build_plan/native_dynamic_package_plan.rs`（`plugins/<dir>/plugin.toml` + `native_dynamic_package.toml` + ABI v3 契约表 + 目录去重诊断）、`native_plugin_loader/native_plugin_load_manifest.rs`（`NativePluginLoadManifest` + ABI 契约），loader 在 `builtin/runtime_modules/plugin_modules/loader.rs`。
- **导出 CLI 阶段在**：`tools/zircon_export/` 已有完整 NativeDynamic 阶段（`native_build.py` 真编译、`native_signing.py` 签名、`native_dynamic.py`、`pipeline_report_native_dynamic_loader_manifest.py` loader manifest、pack delta、platform bundle）。
- **staged 构建已区分形态**：`tools/zircon_build.py` 的 `PLUGIN_CARRIERS = ("all", "native_dynamic", "rlib_static")`、`is_native_dynamic`/`native_dynamic_crates`/`rlib_static_crates`、`--plugins`、`PLUGIN_LOAD_MANIFEST = "plugins/native_plugins.toml"`。

主要缺口：

| # | 缺口 | 规范条目 | 证据路径 |
|---|------|---------|---------|
| B1 | **唯一独立可构建插件是 fixture**：仅 `native_dynamic_fixture` 是 `crate-type = ["cdylib"]`；其余 ~83 个一方插件均 `rlib`，无 cdylib 形态 | §标准产物 | `grep crate-type zircon_plugins/**/Cargo.toml` 仅命中 `native_dynamic_fixture/native/Cargo.toml` |
| B2 | **dist 形态依赖边界未解耦**：runtime crate 直链 `zircon_runtime = { path = "../../../zircon_runtime" }`，无法脱离源码树编译 | §依赖边界 | `animation/runtime/Cargo.toml`（`zircon_runtime` path dep）；对比 `native_dynamic_fixture/native/Cargo.toml` 仅依赖 `zircon_plugin_sdk` `native` feature |
| B3 | **注册语义不可跨 ABI 编组**：ABI v3 仅暴露 `invoke_command` + bridge method；`RuntimePlugin::register` 的 register_module / register_system / register_resource / register_event / 扩展点注册无 ABI 编组通道，故"系统型"插件（绝大多数）无法做成 cdylib | §注册编组 | `plugin_sdk/src/native.rs:97-107`（`NativePluginBehaviorV3` 仅 invoke/save/restore/unload）；`plugin_sdk/src/registration.rs`（in-tree 注册经 `zircon_runtime` 类型） |
| B4 | **无单源双投影**：manifest 是 descriptor 派生（12-M1），但 in-tree rlib 与 dist cdylib 没有统一声明同时喂两形态 | §双形态单源 | `runtime_exports.rs`（`runtime_plugin_exports!` 只投影 in-tree）；fixture 的 cdylib 与一方 rlib 各写各的 |
| B5 | **无每插件构建子命令**：`zircon_build.py --targets plugins` 是 staged 全量布局；`zircon_export` 是 profile 级整包导出；都没有"单插件独立产物 + 独立 target dir + 独立 lockfile 语义" | §构建命令 | `tools/zircon_build.py`（target 粒度）、`tools/zircon_export/cli.py`（profile 粒度） |
| B6 | **无依赖边界 guard**：没有审计断言 dist crate 的依赖闭包不含 `zircon_runtime`，无法防回归 | §强制机制 | `tools/plugin_structure_audits/` 现有 manifest/skeleton/registration/capability owner，无 dependency_boundary owner |
| B7 | **资产子包未按插件切分**：zrpack（09-M2）是发行整包级；插件自带资产（shader/preset）无 per-plugin 子包随 cdylib 分发 | §资产子包 | 09 §3.3 zrpack 为 profile 级；`native_dynamic_package_plan.rs` 产物目录无资产入口 |
| B8 | **`RuntimePluginId` 封闭枚举**：第三方/独立插件自带 id 必须改引擎核心（与 12-S12/D6 同根） | 架构 | `builtin/runtime_modules/ids/plugin_id.rs`（core 封闭 enum）；12 计划 M5/T2 在途 |

> B3 是本计划的**最深依赖**：它决定"每插件独立构建"的覆盖面——在 B3 闭合前，只有 command/bridge 型插件能做成 cdylib；闭合后，系统型插件才能全量独立构建。B3 必须与 [Plugins 01](01-plugin-architecture-core.md) ABI v3 / register 通道、[Plugins 11](11-plugin-call-bridge.md) bridge dense 通道协同推进，不在本计划内独立发明第二套注册模型。

## 3. 架构设计

### 3.1 双形态单源（解决 B1/B2/B4）

一份插件声明，投影两个产物形态，共享一份纯逻辑：

```
<plugin>/
  plugin.toml          # 单源 manifest（12-M1 schema），新增 [distribution] 段（见 §3.4）
  runtime/
    Cargo.toml         # crate-type = ["rlib", "cdylib"]
                        # zircon_runtime = { path = "...", optional = true }  ← 仅 embed feature 拉入
                        # [features] embed=[dep:zircon_runtime]  dist=[zircon_plugin_sdk/native]
    src/
      lib.rs           # 薄 façade
      plugin.rs        # impl RuntimePlugin::register（cfg(feature="embed")，in-tree 注册 owner）
      dist.rs          # cfg(feature="dist")：ABI v3 导出（SDK 宏），调用 backend
      capability.rs    # capability 单源（12-M4）
      backend/         # 纯逻辑 owner：仅依赖 zircon_plugin_sdk + zircon_runtime_interface，禁依赖 zircon_runtime
      systems/         # ECS 系统（embed 形态注册进调度图；dist 形态经 §3.3 编组）
      tests/
  editor/              # 镜像同形态
```

- **embed 形态**（`--features embed`，默认）：`rlib`，`plugin.rs` 走 `plugin_sdk::registration::RuntimePluginRegistrationBuilder`（12-M3），静态链进 `zircon_app`，零 FFI——LibraryEmbed 性能路径。
- **dist 形态**（`--no-default-features --features dist`）：`cdylib`，`dist.rs` 用 `native_command_plugin_v3!` 等 SDK 宏导出 `zircon_native_plugin_descriptor_v3` + entry，依赖闭包**不含 `zircon_runtime`**——NativeDynamic 可分发路径。
- **铁律**：`backend/` 与 `capability.rs` 不得 `use zircon_runtime::*`；只能用 `zircon_plugin_sdk`（再导出 `zircon_runtime_interface`）。`plugin.rs`/`systems/` 中触碰 `zircon_runtime` 的代码一律 `#[cfg(feature = "embed")]`。这是 B2/B6 的可审计基线。
- **备选**（逻辑无法干净 feature-gate 时）：拆独立 `<plugin>/dist/` cdylib crate 包裹 `backend/`，作为 fallback 而非首选，避免三 crate 样板膨胀。

### 3.2 依赖边界（解决 B2/B6）

dist 产物的依赖闭包白名单（编译期 guard）：

| 允许 | 禁止 |
|---|---|
| `zircon_plugin_sdk`（`native`/`dist` feature） | `zircon_runtime`（path 或任何形态） |
| `zircon_runtime_interface`（经 SDK 再导出，稳定 ABI/DTO） | `zircon_editor` / `zircon_app` / `wgpu` / `slint` / `winit` |
| 纯第三方算法库（kira、jolt、recast 等） | runtime 世界引用、trait object、wgpu/slint 对象（与 E8 边界规则同源） |

- guard：`cargo metadata --manifest-path zircon_plugins/Cargo.toml -p <dist crate> --features dist --no-default-features --no-deps` 解析依赖，断言 `zircon_runtime` 不在闭包；新增审计 owner `tools/plugin_structure_audits/dependency_boundary.py`，字段 `dist_dependency_boundary_violations`、`dist_capable_plugin_count`。

### 3.3 注册跨 ABI 编组（解决 B3，与 01/11 协同）

`RuntimePlugin::register` 在 in-tree 形态直接调 runtime registry；dist 形态需把"插件想注册什么"序列化为 ABI-safe 声明，由宿主在加载期回放进真实 registry：

- **声明序列化**：插件在 dist 形态导出一份 **registration manifest**（module / system anchors / resources / events / 扩展点贡献 / capability），经 `NativePluginEntryReportV3.package_manifest_toml` + 新增 registration 段承载（ABI-safe TOML/字节，非 trait object）。
- **行为编组**：系统执行体不跨 FFI 传函数指针进 runtime 调度（生命周期/借用不可控），而是：
  - **command/bridge 型**：沿用 `invoke_command` + `NativePluginBridgeMethodTableV3`（已在），宿主侧 system 调用插件方法（11 的 WeakBridge/StrongBridge dense 通道）。
  - **system 型**：插件声明 system anchor 与读写访问集（`SystemParamAccess`，01 已在），宿主据此在调度图占位，tick 时经 bridge 回调插件执行体——把"系统注册"降解为"宿主拥有的 system + 跨 ABI 调用"。
- **边界**：本计划只定义 registration 段的 ABI schema 与 loader 回放契约；01 负责 register 通道与访问集语义、11 负责 bridge dense 通道。B3 的里程碑（M2）显式标注依赖 01-M2/M4、11-M1/M2，不重复造模型。

### 3.4 manifest `[distribution]` 段（解决 B4/B5，schema 见规范文档）

```toml
[distribution]
forms = ["embed", "dist"]            # 该插件支持的产物形态
default_packaging = ["library_embed", "native_dynamic"]
abi_version = 3                       # 与 ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3 钉
engine_compat = ">=0.1, <0.2"         # 引擎兼容区间，loader 加载期校验
dist_crate = "zircon_plugin_<plugin>_runtime"   # cdylib crate
descriptor_symbol = "zircon_native_plugin_descriptor_v3"
runtime_entry = "zircon_plugin_<plugin>_runtime_entry_v3"
editor_entry = "zircon_plugin_<plugin>_editor_entry_v3"   # 可选
assets = ["assets/**"]                # 随包资产（进 per-plugin zrpack 子包，§3.6）
```

- 静态生成纪律延续 12-M1：非 native 的 `[distribution]` 仍由 descriptor `package_manifest()` 投影（`@generated`），不手写漂移。

### 3.5 每插件独立构建命令（解决 B5）

新增子命令（演进现有 CLI，不另起炉灶）：

```bash
# 单插件独立构建：独立 target dir、独立产物目录、不全量编译 workspace
python -m tools.zircon_export plugin build <id> \
    --form dist --platform windows-x86_64 --mode release \
    --out E:/zircon-build/plugins --target-dir <drive>/cargo-targets/zircon-plugin-<id>

# 产物：<out>/<id>/{ <id>.dll, plugin.toml, native_dynamic_package.toml, <id>.zrpack(可选), <id>.sig }
```

- 复用 `tools/zircon_export/native_build.py`（真编译）+ `native_signing.py`（签名）+ `native_dynamic_package_plan.rs` 报告契约；`plugin build` 是单插件切片入口，profile 级整包导出（09）调用同一编译/签名底座。
- `zircon_build.py` 的 `rlib_static`/`native_dynamic` carrier 与本命令共用 crate 形态判定（`is_native_dynamic` 升级为读 `[distribution].forms`）。
- 独立性：单插件构建用独立 `--target-dir` 与 dist feature 集，CI 可对每插件独立 job 编译，不被 workspace 其他 crate 牵连（B5）。

### 3.6 per-plugin 资产子包（解决 B7，复用 09 zrpack）

- 插件 `[distribution].assets` 列出的资产经 09-M2 的 zrpack writer 打成 `<id>.zrpack`（内容寻址 chunk，确定性写入），随 cdylib 落 `plugins/<id>/`；与发行整包 zrpack 共用 `ZrPackManifest`/`ZrChunkEntry` DTO（09 §3.3 / 07 §3.7）。
- loader 加载插件时挂载其 zrpack 到资产虚拟路径 `plugin://<id>/...`；裁剪/去重沿用 09 规则，禁止静默裁剪。

## 4. 模块文件树

```
zircon_plugins/plugin_sdk/src/
  native.rs                 [改造] 增 registration 段 ABI schema（§3.3）+ dist entry helper 宏
  dist.rs                   [新增] dist 形态投影 helper（单源喂 cdylib 导出）
  registration.rs           [改造] embed/dist 双形态 registration builder 收口
zircon_plugins/<plugin>/
  plugin.toml               [改造] 增 [distribution] 段
  runtime/Cargo.toml        [改造] crate-type rlib+cdylib + embed/dist features + optional zircon_runtime
  runtime/src/dist.rs        [新增] cfg(dist) ABI 导出 owner
  runtime/src/backend/       [改造] 收束依赖边界（剥离 zircon_runtime 直依赖）
zircon_runtime/src/plugin/
  native_plugin_loader/      [改造] registration 段回放 + engine_compat/abi 协商诊断
  export_build_plan/native_dynamic_package_plan.rs  [改造] 产物目录增 zrpack 资产入口
tools/zircon_export/
  cli.py                    [改造] 增 `plugin build <id>` 子命令
  native_build.py           [改造] 单插件 dist feature 编译入口
tools/zircon_build.py       [改造] carrier 形态判定读 [distribution].forms
tools/plugin_structure_audits/
  dependency_boundary.py    [新增] dist 依赖闭包白名单 guard
docs/zircon_plugins/
  plugin-standalone-build.md [新增] 独立构建/分发唯一规范权威
```

## 5. 里程碑（任务级执行蓝本）

切片期 `cargo check --manifest-path zircon_plugins/Cargo.toml -p <crate> --features dist --no-default-features --locked`；里程碑末进测试 + `cargo fmt --all --check` + `python tools/audit_plugin_structure.py --json`。

| 里程碑 | 任务 | 改动文件（代表） | 依赖 | 验收命令 / 测试函数 |
|---|---|---|---|---|
| **M1 双形态骨架 + 依赖边界** | T1 规范文档 owner | `docs/zircon_plugins/plugin-standalone-build.md` | 12-M1/M2 | 人工 review + 链接可达 |
| | T2 `crate-type rlib+cdylib` + `embed`/`dist` feature + optional `zircon_runtime`（代表插件先行） | 代表插件 `runtime/Cargo.toml`、`src/{plugin.rs,dist.rs}` | 12-M3 | `cargo check -p <crate> --features dist --no-default-features` 不拉入 `zircon_runtime` |
| | T3 依赖边界 guard | `tools/plugin_structure_audits/dependency_boundary.py` | T2 | `plugins_13_dist_dependency_boundary_clean`（`dist_dependency_boundary_violations = 0`） |
| | T4 `plugin.toml` 增 `[distribution]` 段 + `@generated` 投影 | manifest schema 校验器、各 `plugin.toml` | M1-T1 | `plugins_13_distribution_section_uniform` |
| **M2 注册跨 ABI 编组** | T1 registration 段 ABI schema（module/system anchor/resource/event/扩展点/capability） | `plugin_sdk/src/native.rs`、`native_plugin_loader/` | 01-M2/M4 | `native_dynamic_registration_manifest_round_trips` |
| | T2 system 型插件经 bridge 回放（与 11 dense 通道对齐） | `plugin_sdk/src/{native,registration}.rs`、对齐 11 | 11-M1/M2，T1 | `dist_system_plugin_loads_and_ticks_via_bridge` |
| | T3 dist entry helper 宏（一文件导出 cdylib） | `plugin_sdk/src/dist.rs`、`native.rs` 宏 | T1 | `dist_plugin_one_file_export_compiles` |
| **M3 每插件独立构建命令** | T1 `zircon_export plugin build <id>`（独立 target/产物/形态） | `tools/zircon_export/cli.py`、`native_build.py` | M1 | `plugin_build_emits_isolated_package_dir` |
| | T2 carrier 形态判定读 `[distribution].forms` | `tools/zircon_build.py` | M1-T4 | `zircon_build_classifies_forms_from_manifest` |
| | T3 可重复构建：同输入 byte 相同（双跑比对） | `native_build.py`、CI | T1 | `plugin_dist_build_is_byte_reproducible` |
| **M4 产物包 + 兼容性协商** | T1 产物目录增 per-plugin zrpack 资产子包 | `native_dynamic_package_plan.rs`、09 zrpack writer | 09-M2 | `native_dynamic_package_includes_plugin_zrpack` |
| | T2 loader 加载期协商 `abi_version`/`engine_compat`，不匹配出诊断 | `native_plugin_loader/` | M2-T1 | `loader_rejects_incompatible_abi_with_diagnostic` |
| | T3 签名/hash + load manifest 汇编 | `native_signing.py`、`native_plugin_load_manifest.rs` | T1 | `native_plugin_load_manifest_assembles_signed_entries` |
| **M5 全量 rollout + CI 矩阵** | T1 touch-it-conform-it：各插件随 02–11 能力波次迁双形态 | 各插件，随其能力波次 | M1–M4 | `dist_capable_plugin_count` 递增至全量；`plugin_skeleton_gate.migration_debt_count → 0` |
| | T2 每插件独立构建 CI job 矩阵 | `.github/workflows/ci.yml` | M3 | 每插件独立 `cargo build --features dist` 绿 |
| | T3 `RuntimePluginId` string-newtype（与 12-M5/T2 / D6 合流） | `builtin/runtime_modules/ids/plugin_id.rs` | 12-M5 | 第三方插件自带 id 不改引擎核心 |

> 波次：M1–M2 = **波次零（结构前置）**，与 12 同窗口（双形态骨架是 12 骨架的发行维扩展）；M3–M4 进 plugins index §2 波次四（与 09-M1/M2 同窗口共享 zrpack/CLI 底座）；M5 随各能力波次 touch-it-conform-it。

## 6. 验收命令

里程碑末全绿：

```bash
# 双形态编译（代表插件）
cargo check --manifest-path zircon_plugins/Cargo.toml -p <crate> --locked                       # embed (default)
cargo check --manifest-path zircon_plugins/Cargo.toml -p <crate> --no-default-features --features dist --locked
# 依赖边界：dist 闭包不含 zircon_runtime
cargo metadata --manifest-path zircon_plugins/Cargo.toml --format-version 1 --no-deps --locked
python tools/audit_plugin_structure.py --json     # dist_dependency_boundary_violations / dist_capable_plugin_count
# 独立产物构建
python -m tools.zircon_export plugin build <id> --form dist --platform windows-x86_64 --out <out>
# 整体回归
cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --all-targets --locked
cargo fmt --all --check
```

并保持 12 四源一致性（plugin.toml / capability.rs / descriptor / workspace member）+ 本计划新增的 `[distribution]` 段与 dist crate 形态一致性。

## 7. 风险

- **R1 注册编组复杂度（B3/M2）**：system 型插件经 bridge 回放有 tick 期 FFI 调用开销与生命周期约束；缓解——优先 command/bridge 型与 importer 型先 dist 化，system 型在 01/11 dense 通道就绪后迁，过渡期允许只支持 embed。
- **R2 双形态编译矩阵翻倍**：每插件 embed+dist 两套构建增 CI 时间；缓解——per-plugin 独立 job 并行 + 独立 target dir + sccache（CLAUDE.md fast-build 约定）。
- **R3 依赖边界回归**：开发者无意在 `backend/` 引入 `zircon_runtime`；缓解——M1-T3 guard 编译期阻断，纳入 gate。
- **R4 与 09 整包导出重叠**：`plugin build` 与 profile 导出须共用编译/签名/zrpack 底座，避免二套实现漂移；缓解——M3-T1 显式复用 `native_build.py`/`native_signing.py`/zrpack writer。

## 8. 与既有计划的关系

- 继承 09（三路径、NativeDynamic、zrpack、export-templates、CLI 阶段机）的 NativeDynamic 路径并把它从 profile 级整包升格为 per-plugin 独立路径；09 的整包导出调用本计划同一底座。
- 继承 12（统一 manifest / 唯一骨架 / capability 单源 / plugin_sdk builder）并扩出"发行维"：双形态骨架、`[distribution]` 段、依赖边界 guard。
- 依赖 01（ABI v3 / register 通道 / 访问集）与 11（bridge dense 通道）闭合 B3；本计划不另立第二套注册或调用模型。
- B8（`RuntimePluginId` 封闭枚举）与 12-S12/D6 同根，合流在 M5/T3。
