---
related_code:
  - zircon_plugins/plugin_sdk/src/native.rs
  - zircon_plugins/plugin_sdk/src/registration.rs
  - zircon_plugins/plugin_sdk/src/runtime_exports.rs
  - zircon_plugins/native_dynamic_fixture/plugin.toml
  - zircon_plugins/native_dynamic_fixture/native/Cargo.toml
  - zircon_plugins/native_dynamic_fixture/native/src/lib.rs
  - zircon_plugins/animation/runtime/Cargo.toml
  - zircon_runtime_interface/src/plugin_api.rs
  - zircon_runtime/src/plugin/export_build_plan/native_dynamic_package_plan.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_load_manifest.rs
  - tools/zircon_build.py
  - tools/zircon_export/native_build.py
  - tools/zircon_export/native_signing.py
plan_sources:
  - docs/plans/zircon_plugins/13-standalone-plugin-build.md
  - docs/plans/zircon_plugins/09-export-publishing.md
  - docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md
  - docs/plans/engine-code-structure-convention.md
doc_type: module-detail
status: planned
---

# 插件独立构建与分发规范（Plugin Standalone Build & Distribution）

> 本文是 ZirconEngine 插件**独立构建与可分发动态包**的唯一规范权威，由 [Plugins 13](../plans/zircon_plugins/13-standalone-plugin-build.md) 落地、[引擎结构规范 §6](../plans/engine-code-structure-convention.md) 引用。与 [`plugin-manifest-schema.md`](plugin-manifest-schema.md)（manifest schema）、[`plugin-crate-skeleton.md`](plugin-crate-skeleton.md)（crate 骨架）、[`plugin-sdk.md`](plugin-sdk.md)（SDK API）配套。
>
> 状态：planned（规范定稿；落地随 Plugins 13 M1–M5）

## 1. 设计原则

- **双形态单源**：一份插件声明（manifest + backend 逻辑）投影两种产物——`embed`（in-tree `rlib`，静态链接，零 FFI）与 `dist`（`cdylib`，ABI-only，可分发可热更）。两形态不复制逻辑。
- **依赖边界硬约束**：`dist` 产物的依赖闭包**只含稳定 ABI**（`zircon_plugin_sdk` → `zircon_runtime_interface`），**禁含 `zircon_runtime` / `zircon_editor` / `zircon_app` / `wgpu` / `slint` / `winit`**（与结构规范 §7.5 E8 边界白名单同源）。可独立于引擎源码树编译。
- **稳定 ABI 唯一通道**：跨 cdylib 边界只传 ABI-safe 值与序列化字节（ABI v3 `repr(C)` 表 + TOML/字节 payload），不传 Rust trait object、wgpu/slint 对象、runtime 世界引用。
- **可重复构建**：同输入（lockfile + 源 + 资产）产出 byte 相同 cdylib 与 zrpack；时间戳清零、路径归一。
- **兼容性显式协商**：产物钉 `abi_version` 与 `engine_compat`；loader 加载期校验，不匹配出结构化诊断而非崩溃。

## 2. 产物形态

| 形态 | crate-type | 依赖 | 注册路径 | 用途 |
|---|---|---|---|---|
| `embed` | `rlib` | `zircon_runtime`（path，behind `embed` feature） | `impl RuntimePlugin::register` + `plugin_sdk::registration` builder | LibraryEmbed 静态链接，发行期性能优化 |
| `dist` | `cdylib` | `zircon_plugin_sdk`(`native`) + `zircon_runtime_interface` | ABI v3 导出（`zircon_native_plugin_descriptor_v3` + entry） | NativeDynamic 可分发包、热更插件 |

- 默认 feature = `embed`；`dist` 形态以 `--no-default-features --features dist` 构建。
- 单 crate `crate-type = ["rlib", "cdylib"]` + feature-gated `zircon_runtime`（`optional = true`）为**首选**；逻辑无法干净 feature-gate 时退化为独立 `<plugin>/dist/` cdylib crate 包裹 `backend/`（fallback）。

## 3. crate 骨架（发行维扩展）

在 [`plugin-crate-skeleton.md`](plugin-crate-skeleton.md) 骨架基础上：

```
<plugin>/runtime/
  Cargo.toml
    # crate-type = ["rlib", "cdylib"]
    # [dependencies] zircon_plugin_sdk = { workspace = true, default-features = false }
    #                zircon_runtime = { path = "...", optional = true }
    # [features] default = ["embed"]
    #            embed = ["dep:zircon_runtime", "zircon_plugin_sdk/runtime"]
    #            dist  = ["zircon_plugin_sdk/native"]
  src/
    lib.rs           # 薄 façade
    plugin.rs        # #[cfg(feature="embed")] impl RuntimePlugin::register
    dist.rs          # #[cfg(feature="dist")]  ABI v3 导出 owner（SDK 宏）
    capability.rs    # capability 单源（禁 use zircon_runtime）
    backend/         # 纯逻辑：仅 zircon_plugin_sdk + zircon_runtime_interface
    systems/         # embed 注册进调度图；dist 经 §6 编组
    tests/
```

- 铁律：`backend/`、`capability.rs` 禁 `use zircon_runtime::*`；触碰 `zircon_runtime` 的代码必须 `#[cfg(feature = "embed")]`。

## 4. manifest `[distribution]` 段

`plugin.toml` 新增可选段（schema 详见 [`plugin-manifest-schema.md`](plugin-manifest-schema.md) §3）：

```toml
[distribution]
forms = ["embed", "dist"]                 # 该插件支持的产物形态
default_packaging = ["library_embed", "native_dynamic"]
abi_version = 3                            # 与 ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3 钉
engine_compat = ">=0.1, <0.2"             # 引擎兼容区间
dist_crate = "zircon_plugin_<plugin>_runtime"
descriptor_symbol = "zircon_native_plugin_descriptor_v3"
runtime_entry = "zircon_plugin_<plugin>_runtime_entry_v3"
editor_entry  = "zircon_plugin_<plugin>_editor_entry_v3"   # 可选
assets = ["assets/**"]                     # 随包资产（进 per-plugin zrpack 子包）
```

- 非 native 插件的 `[distribution]` 由 descriptor `package_manifest()` 投影（`@generated`），不手写漂移（延续 12-M1 生成纪律）。

## 5. 产物包布局

`zircon plugin build <id> --form dist` 产出：

```
plugins/<id>/
  <id>.{dll|so|dylib}            # cdylib，导出 descriptor 符号 + entry
  plugin.toml                     # 包级 manifest（@generated 投影）
  native_dynamic_package.toml     # ABI v3 契约报告（native_dynamic_package_plan.rs 格式）
  <id>.zrpack                     # 可选：随包资产子包（内容寻址 chunk，09 zrpack 格式）
  <id>.sig                        # 可选：签名/hash（native_signing.py）
```

- 目录名由 `package_id` 净化（`native_dynamic_package_directory`：非 `[A-Za-z0-9_-]` → `_`），冲突出诊断。
- loader 经 `NativePluginLoadManifest`（`{ id, path, manifest, package_report, abi }`）收集 `plugins/<id>/`，按 ABI v3 契约加载。

## 6. 注册跨 ABI 编组（dist 形态）

`embed` 形态直接调 runtime registry；`dist` 形态把注册意图序列化、由宿主回放（依赖 Plugins 01 register 通道 + 11 bridge dense 通道）：

- **声明序列化**：dist 插件导出 registration manifest（module / system anchors + 读写访问集 / resources / events / 扩展点贡献 / capability），经 `NativePluginEntryReportV3.package_manifest_toml` + registration 段（ABI-safe TOML/字节）承载。
- **行为编组**：
  - command/bridge 型 → `invoke_command` + `NativePluginBridgeMethodTableV3`（宿主调用插件方法）。
  - system 型 → 插件声明 system anchor + `SystemParamAccess`；宿主据此在调度图占位，tick 时经 bridge 回调插件执行体。不跨 FFI 传裸函数指针进调度。
- **panic 边界**：所有 `extern "C"` 边界（出站导出 + 入站 host 回调）必须 panic guard（`catch_native_callback_panic`），panic 转状态码不跨 FFI（结构规范 §7.5 E7）。

## 7. 构建命令契约

```bash
# 单插件独立构建（独立 target dir、独立产物目录，不全量编译 workspace）
python -m tools.zircon_export plugin build <id> \
    --form dist --platform <triple> --mode release \
    --out <out>/plugins --target-dir <isolated-target-dir>
```

- 复用 `tools/zircon_export/native_build.py`（真编译）+ `native_signing.py`（签名）+ zrpack writer（09-M2）；profile 级整包导出（09）调用同一底座。
- `tools/zircon_build.py` 的 carrier 形态判定（`native_dynamic`/`rlib_static`）升级为读 `[distribution].forms`。

## 8. 校验器与 guard（Plugins 13）

- 依赖边界：`tools/plugin_structure_audits/dependency_boundary.py` 解析 `cargo metadata -p <crate> --features dist --no-default-features --no-deps`，字段 `dist_dependency_boundary_violations`（→ 0）、`dist_capable_plugin_count`。
- `[distribution]` 段一致性：`plugins_13_distribution_section_uniform`。
- 可重复构建：`plugin_dist_build_is_byte_reproducible`（双跑 byte 比对）。
- 兼容性协商：`loader_rejects_incompatible_abi_with_diagnostic`。
- 与 12 四源一致性 guard（plugin.toml / capability.rs / descriptor / workspace member）联合执行。
