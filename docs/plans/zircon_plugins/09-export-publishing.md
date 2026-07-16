# 09 · 跨平台发行能力计划（导出模板 / 工程化输出）

> 状态：工程化细化版 v2 · 优先级：P2 · 前置：[01 插件架构核心](01-plugin-architecture-core.md) M3–M4
> 关联计划：`.codex/plans/Runtime_Editor 最小本体与发行导出插件化设计.md`（三路径模型维持有效）、`.codex/plans/zircon_plugins 全量插件化收敛规划.md`
> 参考实现：Godot export template（预编译模板 + pck 注入 + per-platform export preset）、Unreal target/platform module 模型

## 1. 目标

把发行能力推进到完整的工程化导出系统：三条导出路径（SourceTemplate / LibraryEmbed / NativeDynamic）端到端闭环、平台模板体系、导出 profile 资产化、插件与 feature 选择全程参与裁剪、可重复构建。

## 2. 现状基线（实查）

导出基建比早期"面板雏形 + staged 脚本"的判断厚实得多：

- **profile 与策略** `zircon_runtime/src/plugin/export_profile.rs`：`ExportTargetPlatform`/`ExportPlatformHostKind`/`ExportPlatformResourceStrategy`/`ExportPlatformPluginStrategy`/`ExportPlatformPolicy`/`ExportPackagingStrategy`/`ExportProfile` 全套契约。
- **构建计划与物化** `zircon_runtime/src/plugin/export_build_plan/`：`export_build_plan.rs`、`export_profile_validation.rs`、`from_project_manifest/`（zircon-project.toml → 计划）、`project_manifest_validation/`、**`materialize.rs` + 模板族**（`cargo_manifest_template.rs`/`main_template.rs`/`asset_manifest_template.rs`/`plugin_selection_template.rs`/`native_plugin_load_manifest_template.rs`）——SourceTemplate 的工程生成已部分实现；`native_dynamic_package_plan.rs`、`platform_host_files/`、`export_generated_file.rs`/`export_materialize_report.rs`。
- **编辑器插件** `zircon_plugins/editor_build_export_desktop/`（editor crate + plugin.toml）：导出面板、兼容性诊断、native-dynamic 报告。
- **CI 契约**：`ZR_EXPORT_CONTRACT_PLATFORM` 平台策略契约测试。
- **staged 构建** `tools/zircon_build.py`：targets（hub/editor/runtime/plugins）staged payload、native_dynamic crate 区分（`is_native_dynamic`/`rlib_static_crates`）。

主要缺口：

| # | 缺口 | 证据 |
|---|------|------|
| E1 | 三路径仅 NativeDynamic 有 fixture 级闭环；LibraryEmbed 无 feature 链接矩阵与产物冒烟；SourceTemplate 物化产物未验证可编译 | `materialize.rs` 无 cargo build 验证环节 |
| E2 | 无 zrpack 资产打包（内容寻址 chunk）与依赖闭包裁剪 | 无 pack 模块 |
| E3 | 无平台模板包体系（预编译宿主 + 版本锁定 + Hub 分发） | — |
| E4 | 无导出 CLI：导出逻辑散在编辑器插件与 zircon_build.py，CI 与本地路径不统一 | `tools/zircon_build.py` 仅 staged 布局 |
| E5 | 非 Windows 平台未进矩阵；无确定性构建保证 | `platform_host_files/` |

## 3. 架构设计

### 3.1 导出 profile 资产（`zircon-project.toml` → `[export_profiles.<name>]`）

```toml
[export_profiles.windows-release]
platform = "windows-x86_64"
path = "library_embed"            # source_template | library_embed | native_dynamic
mode = "release"
plugins = ["sound", "physics", "animation", "navigation"]   # 显式裁剪
features = { sound = ["hrtf"], net = ["http", "websocket"] }
asset_filter = "shipping"         # 资产标签裁剪
```

- 解析进现有 `from_project_manifest/` 路径，编译为 `ExportBuildPlan`（现有验证模块输入）；全部验证（插件依赖闭包、capability、平台支持矩阵）在计划期完成并出结构化诊断；features 字段与 01 §3.6 的 optional_features 模型同源（同一 id 命名空间）。

### 3.2 三路径实现

| 路径 | 机制 | 现状 → 目标 |
|------|------|------------|
| **SourceTemplate** | `materialize.rs` 模板族生成独立 Cargo 工程（manifest 注入 + 选定插件 crate 依赖），用户可继续改源码 | 模板生成在 → 补"生成工程 `cargo build` 通过"验证闭环 |
| **LibraryEmbed** | 以 `zircon_app` 为骨架按 profile feature 链接选定插件，产出单一可执行 + zrpack | 缺 → M2 主交付 |
| **NativeDynamic** | 最小宿主 + 插件 cdylib 包（01 §3.7 ABI v3），支持热更插件 | fixture 闭环在（`native_dynamic_package_plan.rs`）→ 升级 ABI v3 + zrpack delta |

- 平台模板体系（Godot 形态）：每平台一个 `export-template` 包——内容清单：预编译宿主骨架（per path×mode）、链接清单、平台胶水（图标/签名占位/打包格式）、`template.toml`（引擎版本锁定 + 内容 hash）；CI 构建，Hub 分发安装（对接 `zircon_hub` Installs 页，下载走 [07](07-net.md) content_download 同一 chunk 协议）。
- 平台矩阵 v1：windows-x86_64、linux-x86_64、macos-aarch64（`ExportTargetPlatform` 现枚举扩展）；预留 `PlatformTargetDescriptor { triple, bundle_format, asset_compression, graphics_backend_constraints }`，web/mobile 进后续池但描述符结构一步到位。

### 3.3 zrpack 资产打包与裁剪（解决 E2，`zircon_runtime/src/asset/pack/` [新增]）

字节级格式定稿（与 [07 Net](07-net.md) §3.7 共享 `ZrPackManifest`/`ZrChunkEntry` DTO，定义在 `framework/net/download.rs`，发行包与热更包共用）：

```
zrpack 文件布局（小端）:
  header:  magic "ZRPK" | format_version: u32 | manifest_offset: u64 | manifest_size: u64
  chunks:  内容寻址数据区（blake3 hash 即 chunk id，重复内容自动去重）
  manifest: ZrPackManifest 序列化（chunk 表 + 资产路径 → chunk 区间索引表）
```

- 裁剪：从场景/入口出发的资产依赖闭包（importer 输出的引用图）+ `asset_filter` 标签；未引用资产剔除并出报告（逐项列出被剔除路径，**禁止静默裁剪**）。
- 确定性：同一输入（lockfile + 资产版本）产出 byte 相同 pack——归一化清单：资产按路径字典序写入、时间戳清零、绝对路径剥离、chunk 排序按 hash。双跑比对进 CI。

### 3.4 导出执行器与 CLI（解决 E4，`tools/zircon_export/` [新增，python 包]）

- 导出在编辑器外进程执行：`tools/zircon_build.py` 演进为 `python -m tools.zircon_export --profile <name>`（编排 cargo + 资产管线），编辑器面板（editor_build_export_desktop）只是该 CLI 的 UI 壳——CI 与本地共用同一路径。
- 阶段状态机：共享顺序为 `Validate → SourceTemplate → NativeDynamic → CompileHost → CookAssets → Pack → PlatformBundle → Report`；Validate 后按 profile strategy 裁剪 SourceTemplate/NativeDynamic/LibraryEmbed 闭包，每阶段产物落盘（`<out>/stages/<stage>/`）可恢复（`--resume-from <stage>`），失败给阶段级诊断；`export_materialize_report.rs` 现报告类型扩展为全阶段报告。

## 4. 模块文件树

```
zircon_runtime/src/plugin/
  export_profile.rs                       [改造] 平台矩阵扩展 + PlatformTargetDescriptor
  export_build_plan/from_project_manifest [改造] export_profiles 节解析 + features 字段
  export_build_plan/materialize.rs        [改造] 生成工程构建验证钩子
zircon_runtime/src/asset/pack/
  {writer,reader,dedup,trim}.rs           [新增] zrpack 读写/去重/依赖闭包裁剪
zircon_runtime/src/core/framework/net/download.rs  [07 计划共享 DTO，本计划消费]
tools/zircon_export/                      [新增] CLI 包（阶段状态机/恢复/报告）
tools/zircon_build.py                     [改造] staged 布局保留，导出编排迁出
zircon_plugins/editor_build_export_desktop/editor/  [改造] 面板改为 CLI 壳 + 向导（M6）
tools/zircon_export/export-templates/                         [新增 CI 产物仓] 每平台 template.toml + 宿主骨架
```

## 状态与产出记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

本子计划产出记录已超过 10 条，具体记录已迁入编号子目录。

- 迁入记录：[`../_archive/zircon_plugins/09/2026-07-09-export-publishing-output-records.md`](../_archive/zircon_plugins/09/2026-07-09-export-publishing-output-records.md)

## 5. 里程碑与任务分解

### M1 Profile 与计划期验证

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M1-T1 | export_profiles 节解析（plugins/features/asset_filter） | from_project_manifest/ | 01-M3 | `profile_with_features_compiles_to_build_plan` |
| M1-T2 | 计划期全验证 + 非法组合拒绝 | export_profile_validation.rs | M1-T1 | `invalid_plugin_combination_rejected_with_diagnostic` 矩阵 |
| M1-T3 | CLI 骨架（Validate 阶段 + 报告） | tools/zircon_export | M1-T2 | CLI 冒烟（profile → 验证报告） |

### M2 LibraryEmbed 闭环

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M2-T1 | feature 链接矩阵（profile → zircon_app feature 集 + 插件 crate 依赖注入） | materialize.rs、CLI CompileHost | M1 | `feature_matrix_links_selected_plugins_only` |
| M2-T2 | zrpack writer/reader + 去重 | asset/pack/ | 07-M6-T1（DTO） | `pack_round_trip`、`duplicate_content_stored_once` |
| M2-T3 | 依赖闭包裁剪 + 剔除报告 | asset/pack/trim.rs | M2-T2 | `unreferenced_asset_trimmed_and_reported` |
| M2-T4 | windows bundle + 启动冒烟 + 确定性双跑 | CLI PlatformBundle | M2-T1..T3 | 导出产物启动到首帧冒烟；`deterministic_pack_double_run_byte_identical` |

### M3 平台模板体系

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M3-T1 | export-template 包格式 + 版本锁定 | tools/zircon_export/export-templates/、CLI | M2 | `template_version_mismatch_rejected` |
| M3-T2 | linux/macos 模板（未签名可运行 bundle） | platform_host_files/、CI | M3-T1 | 三平台 CI 导出矩阵 |
| M3-T3 | Hub 安装对接（content_download 协议） | zircon_hub Installs | 07-M6 | Hub 安装端到端测试 |

### M4 SourceTemplate

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M4-T1 | 生成工程补全（现模板族 + 资产引用）并 `cargo build` 验证闭环 | materialize.rs、CLI | M1 | `materialized_project_builds_and_runs` |

### M5 NativeDynamic 发行

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M5-T1 | ABI v3 插件包导出（native_dynamic_package_plan 升级） | native_dynamic_package_plan.rs | 01-M5 | fixture 升级保绿 |
| M5-T2 | 热更包（zrpack delta：新旧 manifest 的 chunk 差集） | asset/pack/、CLI | M2-T2、M5-T1 | `delta_pack_contains_only_changed_chunks`、插件热更端到端 |

### M6 Editor 面板完善

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M6-T1 | 导出向导（布局 `ai-build-export-layout.png`）+ 阶段进度（CLI 进程输出流式解析）+ 报告视图 | editor_build_export_desktop | M2、[10 规范](10-editor-integration.md) | editor 契约测试；`docs/zircon_plugins/editor-build-export-desktop.md` 更新 |

## 6. 验收命令

```bash
ZR_EXPORT_CONTRACT_PLATFORM=windows cargo test -p zircon_runtime platform_target_policy_matches_host_resource_and_plugin_strategy --locked --verbose
python -m tools.zircon_export --profile windows-release --out E:/zircon-export
python tools/zircon_build.py --targets hub,editor,runtime --out E:/zircon-build --mode debug
cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_editor_build_export_desktop --locked
```

## 7. 风险

- macOS 签名/公证与 Linux 打包格式各有平台债：M3 先交付“未签名可运行 bundle”，签名链路单列后续项。
- 资产依赖闭包要求 importer 输出完整引用图；缺口由 asset 管线侧（`docs/zircon_runtime/asset/management.md` 体系）补齐，导出侧只消费——M2-T3 前需确认引用图覆盖（gltf 子资产/材质纹理引用为高风险点）。
- 确定性 pack 的双跑比对对压缩器版本敏感；压缩算法与版本锁进 `format_version`。

## 8. 附录 · dev 参考源码对位

实现各任务前**必须先读对应参考实现**，导出流程与平台胶水对照真实代码核对，禁止凭空实现：

| 设计点 | 参考源码（已核验存在） | 看什么 |
|--------|----------------------|--------|
| 导出 preset/平台导出器/模板注入（最重要） | `dev/godot/editor/export/`（`editor_export_platform.*`、`editor_export.cpp`、`codesign.*`） | per-platform preset 字段、模板版本匹配校验、pck 注入流程、签名链路的阶段划分——M1/M3 的判例 |
| 各平台打包胶水（bundle 格式/图标/启动器） | `dev/godot/platform/`（windows/linuxbsd/macos 子目录的 export 部分） | 平台 bundle 目录布局、可执行重命名与资源嵌入方式 |
| pck/资产包格式 | `dev/godot/core/io/`（pck_packer/file_access_pack 相关） | 索引+偏移的包布局、运行期挂载——zrpack 自有格式（内容寻址）但挂载形态可借鉴 |
| 模块化目标/构建编排 | `dev/UnrealEngine/Engine/Source/Runtime/`（模块划分形态）+ 仓内 `tools/zircon_build.py` | target×platform 维度的构建矩阵组织 |
