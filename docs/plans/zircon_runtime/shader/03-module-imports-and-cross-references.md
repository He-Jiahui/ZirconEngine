# 计划 SH-03:WGSL 模块 import 与跨引用

## 目标

1. `include` 型 shader 资产以 `import_path` 注册进模块注册表,任意 shader(surface/compute/fullscreen/include)经 `imports` 声明 + WGSL 内 `#include <path>` 跨模块引用。
2. 内嵌 `zr_*` include 与资产级模块统一为同一注册表与同一解析器:依赖图解析、按 id 注入一次去重、环检测诊断。
3. 模块内容哈希参与引用者的变体缓存键:模块改动 → 引用者全部失效重编,支持热重载传播。
4. 符号纪律:模块导出符号须带模块前缀,冲突在 naga 解析层诊断而非链接期报错。

## 现状与差距

- `include_registry.rs`(`graphics/shader/template/include_registry.rs`)只支持 `include_str!` 内嵌的引擎内建模块;`ZShaderDocument.imports` 有 `redirect: AssetReference` 字段位但用户资产间引用未实际打通。
- 无依赖图:include 内再 include 的传递解析、重复注入去重按内建集合硬处理,不支持资产扩展。
- 模块改动不会使引用者磁盘缓存失效(计划 08 已定 `hash = blake3(canonical_string + 全部参与 include 的内容哈希)`,但"全部参与 include"目前只含内建)。
- 插件的 shader 模块(如 shader_graph 产物公共库)无注册入口。

## 参考代码

| 文件 | 应重点阅读 |
|------|-----------|
| `dev/bevy/crates/bevy_shader/src/shader_cache.rs` | `naga_oil::compose::Composer`:模块注册、传递依赖组合与去重、模块可用性检查——本计划解析器的直接 Rust 同类(本仓库自实现,不引入 naga_oil 依赖,但语义对齐) |
| `dev/bevy/crates/bevy_pbr/src/render/pbr_bindings.wgsl` 等 | `#define_import_path` 模块身份声明 + `#import` 消费的组织方式——import_path 命名空间样板 |
| `dev/Graphics/.../core/ShaderLibrary/Common.hlsl` 与 URP `ShaderLibrary/` 目录 | 功能域分文件、include 链分层(Core → CommonMaterial → BRDF)——引擎内建模块库的目录组织样板 |
| 本仓库 `graphics/shader/template/{include_registry,assemble}.rs` | 现有注入去重与拼接顺序;本计划在此扩展为统一注册表 |
| 本仓库 `asset/importer/ingest/import_shader_package.rs` | imports/redirect 的导入面 |

## 目标架构

归属:模块 id/依赖契约在 `core/framework/render/shader/`;注册表与解析器在 `graphics/shader/module_registry/`(由现 `include_registry` 原位演进,硬切换改名);资产侧导入在 `asset/`。

### 模块身份与引用语法

- 模块 id 即 `import_path`(如 `zircon::noise`、`myproj::cloth::common`):`zircon::` 与 `zr_` 保留给引擎与官方插件,项目资产用项目命名空间,importer 校验保留段。缺省值由资产路径确定性推导、显式声明降级为覆写位(推导规则归 SH-05);`self::` 为拼接器保留命名空间(SH-05 生成符号锚点),资产与插件不可占用,且豁免 `imports` 声明义务。
- WGSL 内引用沿用既有 `#include <path>` 语法(与内建 `#include <zr_surface_types>` 同语法,降低心智负担;不引入 `#import` 第二语法)。
- `.zshader` 的 `imports` 列表是**引用意图声明**:WGSL 内 `#include` 的路径必须出现在 imports(或为内建 `zr_*`),否则 import 期诊断——使依赖图对资产管线可见(依赖追踪/热重载/预热清单都读 zshader,不解析 WGSL 文本)。
- `imports` 项支持 `redirect`(既有字段):同一 import_path 可被项目内资产覆盖重定向,用于替换官方模块实现(诊断记录重定向来源)。

### 注册表与解析

```rust
// graphics/shader/module_registry/mod.rs
pub struct ShaderModuleRegistry {
    // 内建: include_str! 静态注册(现 include_registry 迁移);
    // 资产: ResourceStreamer 按 import_path → ShaderAsset(kind=Include) 惰性供给;
    // 插件: 注册 API(shader_graph 公共库、领域扩展模块经 plugin manifest 注册)。
}
impl ShaderModuleRegistry {
    pub fn resolve(&self, root: &[ImportRef]) -> Result<ResolvedModuleSet, GraphicsError>;
}
pub struct ResolvedModuleSet {
    pub ordered_sources: Vec<ResolvedModule>, // 拓扑序,注入一次去重
    pub content_hash: u64,                    // 全部参与模块内容的 blake3 摘要
}
```

- 解析:从根 shader 的 imports 出发 BFS 传递闭包 → 拓扑排序(依赖先注入)→ 去重;环检测报诊断(含环路径)。
- `content_hash` 交给计划 08 磁盘缓存哈希(其"全部参与 include 的内容哈希"项),并进内存变体键失效比较——模块热改 → ResourceStreamer 报 revision 变化 → 引用者变体重建(沿用材质 revision 通道的传播形态)。
- include 型模块自身不做 naga 独立验证(片段可能依赖引用者上下文),但 import 期做词法级检查:禁止 entry point、禁止 `@group` 声明(模块不得私设绑定,绑定只能来自生成模块与引擎 ABI)、导出符号前缀检查。

### 符号纪律

- 模块顶层符号(fn/struct/const)须以模块尾段名为前缀(`noise_fbm`、`NoiseSettings` 风格按小写函数/大写类型),importer 警告不强制;真正的强制在拼接后 naga 解析:重复符号诊断报"符号 × 两来源模块"。
- 引擎保留前缀(`zr_`/`ZR_OPT_`/`ZrMaterial*`)撞名直接拒绝(index 全局约定 5)。

## 里程碑

### SH03-M1 统一注册表与资产模块导入

实施切片:
1. `include_registry` → `module_registry` 原位演进:内建注册迁移、依赖图解析/拓扑/去重/环检测、`ResolvedModuleSet` 与 content_hash;`assemble.rs` 改走 resolve。
2. `kind=include` 资产导入与 ResourceStreamer 供给;imports 声明与 WGSL `#include` 一致性校验;保留段与词法检查。

测试阶段:
- `cargo check -p zircon_runtime --lib --locked`;`cargo test -p zircon_runtime shader_module --locked`
- 验收证据:三层依赖(surface → A → B)拓扑注入一次;环诊断;undeclared include 诊断;内建路径回归(现有模板拼接产物逐字节不变)。

### SH03-M2 失效传播与插件注册口

实施切片:
1. content_hash 接入计划 08 磁盘哈希与内存失效;模块热改传播(编辑器改 include → 引用者重编)链路。
2. 插件模块注册 API(经 plugin manifest,复用 permutation manifest 的注册形态);redirect 覆盖链与诊断。

测试阶段:
- `cargo test -p zircon_runtime shader --locked` + 热重载集成用例
- 验收证据:改模块一字节 → 引用者磁盘键变化、未引用者不失效;redirect 生效且诊断记录来源;插件注册模块可被项目 surface shader 引用。

## 测试与验收清单

- `render_shader_module_resolve_*`:拓扑/去重/环/undeclared。
- `render_shader_module_hash_*`:失效传播精确性(引用者失效、旁观者不失效)。
- `render_shader_module_lexical_*`:entry point/@group/保留前缀拒绝。
- `render_product_cross_module_material`:跨模块引用的材质端到端产物。

## 状态与产出记录

| 日期 | 里程碑 | 状态 | 完成项目 | 验证与证据 | 后续 |
|---|---|---|---|---|---|
| 2026-07-03 | SH03-M1 统一注册表与资产模块导入 | runtime 主路径完成，focused tests 未全闭合 | `graphics/shader/template/include_registry.rs` 已硬切换为 `module_registry.rs`；内建 include 与资产 include 统一走 `ShaderModuleRegistry`，支持传递依赖拓扑、注入去重、content_hash、环诊断和 unknown module 诊断；`core/framework/render/shader/module_import.rs` 提供 `#include <...>` 扫描/剥离与 `self::`/`zr_*` 分类；模板 assembly 在 surface/GBuffer/TAA 路径展开用户模块并剥离源码内 include 指令；ResourceStreamer 递归收集 include 型 shader 的 `import_path`/source 并供给模板请求；importer 增加 imports 与 WGSL include 一致性诊断、include module 禁 entry point/禁 `@group`/禁保留前缀的词法诊断。 | `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh02-m2 --message-format short --color never` 于 2026-07-03 通过；vampire 截图导出测试通过；模板/ABI 单测 harness 编译超时，未计通过；`git grep -n "include_registry" -- zircon_runtime/src` 无生产源码命中。 | 仍需补跑 `shader_module` focused tests、热重载传播测试、插件注册 API、redirect 覆盖诊断；历史状态行可保留旧 owner 名称作为已迁移背景，新增文档须使用 `module_registry`。 |
| 2026-07-03 | SH03-M2 失效传播与插件注册口 | asset-root include 预热失效传播完成；插件注册口后续已在 2026-07-04 行关闭 | 模块 include 的 content_hash 已进入 template assembly 的 include content hash 列表，可参与现有磁盘 shader hash；引用资产依赖会通过 material/shader dependency set 基础链路进入加载。`zircon_shader_prewarm` asset-root 扫描现在保留 `.zshader` 的 kind/import_path/imports，在生成 variants 前由 `manifest/module_dependencies.rs` 解析同 root 下 include 模块依赖，并把模块内容 hash 混入引用者 request 的 `include_content_hashes` 与 `ShaderVariantKey.material_revision`；无 include 依赖的旁观者 shader revision 保持不变。回归用例下沉到 `manifest/tests/module_dependencies.rs`，父测试文件继续只保留原 10 个 manifest 合约用例。 | `rustfmt --edition 2021 --check --config skip_children=true zircon_runtime/src/bin/zircon_shader_prewarm/manifest.rs zircon_runtime/src/bin/zircon_shader_prewarm/manifest/module_dependencies.rs zircon_runtime/src/bin/zircon_shader_prewarm/manifest/revision.rs zircon_runtime/src/bin/zircon_shader_prewarm/manifest/tests.rs zircon_runtime/src/bin/zircon_shader_prewarm/manifest/tests/module_dependencies.rs` 通过；静态结构等价检查通过：`manifest.rs` 762 行、`manifest/tests.rs` 738 行、`manifest/module_dependencies.rs` 77 行、`manifest/tests/module_dependencies.rs` 97 行，父测试文件 `#[test]` 计数为 10 且文档/status 镜像包含新 child owner；`cargo test -p zircon_runtime --bin zircon_shader_prewarm shader_prewarm_asset_root_manifest_tracks_imported_include_module_revisions --locked --jobs 1 --target-dir F:\cargo-targets\zircon-runtime-shader-sh03-readiness-0704b --message-format short --color never -- --nocapture --test-threads=1` 通过 1/1；`cargo test -p zircon_runtime --bin zircon_shader_prewarm shader_prewarm_asset_root_manifest --locked --jobs 1 --target-dir F:\cargo-targets\zircon-runtime-shader-sh03-readiness-0704b --message-format short --color never -- --nocapture --test-threads=1` 通过 10/10，只有既有 warning；`runtime_15_shader_prewarm_manifest_tests_are_folder_backed` 的 Cargo lib-test wrapper 本轮因 Windows 大 lib-test 编译超时/后续空转未取得计数结果，不计通过。 | 此行仅保留 asset-root prewarm 切片；插件 manifest 与 source-only 项目扫描依赖传播见 2026-07-04 行。redirect 覆盖链产品诊断已由 2026-07-04 产品侧 ResourceStreamer 行聚焦关闭；RenderDoc/product capture 与更广 product/perf sweep 仍待后续切片；focused product material-pass 二次启动 miss=0 已由 SH04 用例关闭。 |
| 2026-07-04 | SH03-M2 redirect 来源诊断补口 | 已完成聚焦验证 | `ShaderImportReadiness` 新增只读 `source_diagnostic`，redirect 行会记录 declared import path 与 redirect locator；source-only import 保持 `None`，不改变 readiness ready/fail 判定、依赖计数、ResourceStreamer 供给、模板拼接或渲染行为。 | `rustfmt --edition 2021 --check --config skip_children=true zircon_runtime/src/asset/assets/shader/readiness.rs zircon_runtime/src/asset/tests/assets/shader_readiness.rs` 通过；`git diff --check` scoped 到同两文件通过，仅报告既有 LF/CRLF 提示；首次 `E:\cargo-targets\zircon-runtime-shader-sh03-readiness-0704` 运行在 `proc-macro2` build-script 链接阶段遇到 MSVC `link.exe` exit code 1318，未编译项目代码且不计结果；改用 `F:\cargo-targets\zircon-runtime-shader-sh03-readiness-0704b` 后 `cargo test -p zircon_runtime --lib shader_readiness_reports_import_rows_without_blocking_source_only_imports --locked --jobs 1 --target-dir F:\cargo-targets\zircon-runtime-shader-sh03-readiness-0704b --message-format short --color never -- --nocapture --test-threads=1` 通过 1/1，只有既有 warning。 | redirect 覆盖链材质 readiness 诊断见 2026-07-04 新行；RenderDoc/product capture 与更广 product/perf sweep 仍待后续产品化切片；focused miss=0 已由 SH04 补跑关闭。 |
| 2026-07-04 | SH03-M2 插件 manifest 模块注册口 | API/工具链/prewarm 与 Rust exact tests 已完成聚焦验证 | 插件 manifest 新增 `[[shader_permutation.shader_modules]]`，Rust manifest 类型 `PluginShaderModuleManifest`、builder `.with_shader_module(...)` 和 TOML roundtrip 用例已接入；`tools/zircon_build_plugin_shader_descriptors.py` 从插件包内相对 `.zshader`/`.wgsl` 源收集 `import_path`、`source` 与 64 位十六进制内容 hash；`PluginPackage.shader_modules`、`generated_shader_permutation_registry_document(...)` 与导出契约校验会写出/校验 `shader_modules`，即使插件只有模块没有自定义 geometry/shading id 也会生成 registry；插件结构审计放行并校验 `shader_permutation.shader_modules` 的 namespace import path、包内 source 路径和重复 import path；`zircon_shader_prewarm` registry overlay 现在合并 `shader_modules` 到外部 include module hash 表，asset-root manifest 扫描用本 root include 模块覆盖外部同名模块，外部模块 hash 作为 fallback 混入引用者 `include_content_hashes` 和 `material_revision`。 | Python 回归通过 52/52；`python -m py_compile` 通过；Rust fmt 通过；`cargo check -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir E:\cargo-targets\zircon-shader-plugin-modules-check --jobs 1 --message-format short --color never` 通过，仅既有 warning；直接运行生成的 test binary `E:\cargo-targets\zircon-shader-plugin-modules-check\debug\deps\zircon_shader_prewarm-040b25e80c0903c6.exe --list` 确认新增 tests 存在，随后 exact 执行 `manifest::permutation_registry::tests::shader_prewarm_permutation_registry_merges_shader_modules` 通过 1/1、`manifest::tests::module_dependencies::shader_prewarm_asset_root_manifest_tracks_registry_shader_module_revisions` 通过 1/1。截图放置复核仍通过：`docs/tests/runtime/shader/runtime_shader_material_sphere_offscreen_20260703.png` 与 `runtime_shader_material_vampire_offscreen_20260703.png` 位于 docs 目录，同名扫描 under repo `target`/`E:\cargo-targets`/`F:\cargo-targets`/`D:\cargo-targets` 无命中。 | redirect 覆盖链产品诊断已由 2026-07-04 产品侧 ResourceStreamer 行聚焦关闭；RenderDoc/product capture 与更广 product/perf sweep 仍待后续切片；focused miss=0 已由 SH04 补跑关闭。 |
| 2026-07-04 | SH03-M2 source-only import 热重载依赖传播 | 已完成聚焦验证 | `ProjectManager::scan_and_import` 在 artifact records 可读后通过 `scan_and_import/shader_import_dependencies.rs` 建立唯一 project include-module `import_path -> AssetUri` 索引，并把 source-only `.zshader [[imports]]` 命中的 include 模块追加进引用 shader 的 `ResourceRecord.dependency_ids`；redirect import 继续走显式依赖，内建/生成模块跳过，重复 import_path 不自动绑定以避免错误依赖。 | `rustfmt --edition 2021 --check --config skip_children=true zircon_runtime\src\asset\project\manager\scan_and_import.rs zircon_runtime\src\asset\project\manager\scan_and_import\shader_import_dependencies.rs zircon_runtime\tests\shader_import_dependency_contract.rs` 通过；`git diff --check` scoped 通过，仅既有 LF/CRLF 提示；`cargo check -p zircon_runtime --lib --no-default-features --features target-server --locked --jobs 1 --target-dir E:\cargo-targets\zircon-shader-live-import-deps-check --message-format short --color never` 通过；`cargo test -p zircon_runtime --test shader_import_dependency_contract --no-default-features --features target-server --locked --jobs 1 --target-dir E:\cargo-targets\zircon-shader-live-import-deps-check --message-format short --color never -- --nocapture --test-threads=1` 通过 1/1，证明引用者依赖 include 模块、旁观者不失效、返回的 imported records 同步携带依赖。 | editor project-sync refresh 精确 Cargo 结果、RenderDoc/product capture 与更广 product/perf sweep 仍待后续切片；redirect 覆盖链产品诊断与 focused miss=0 已分别由产品侧 ResourceStreamer 行和 SH04 补跑关闭。 |
| 2026-07-04 | SH03-M2 redirect 覆盖链材质 readiness 诊断 | 已完成聚焦验证 | `MaterialAsset::readiness_report_with_shader_contract(...)` 现在会检查 `ShaderAsset.dependencies` 中的 redirect shader import 依赖；当 redirect include 模块无法解析时，材质 readiness 追加 `UnresolvedShaderReference` 与 shader fallback usage，避免缺失模块只在后续模板拼接或 pipeline 阶段暴露。 | `rustfmt --edition 2021 --check --config skip_children=true zircon_runtime\src\asset\assets\material\material_asset.rs zircon_runtime\src\asset\tests\assets\material\shader_readiness.rs zircon_runtime\tests\material_shader_redirect_dependency_contract.rs` 通过；`cargo check -p zircon_runtime --lib --no-default-features --features target-server --locked --jobs 1 --target-dir E:\cargo-targets\zircon-shader-redirect-diagnostics-check --message-format short --color never` 通过；`cargo test -p zircon_runtime --test material_shader_redirect_dependency_contract --no-default-features --features target-server --locked --jobs 1 --target-dir E:\cargo-targets\zircon-shader-redirect-diagnostics-check --message-format short --color never -- --nocapture --test-threads=1` 通过 2/2，覆盖内存合成依赖与 `ProjectManager::scan_and_import` 后真实 imported artifact 链路；库测试 wrapper 的同名 lib-test 尝试因整库 test harness 编译超时未计数。 | 产品侧 resolved/missing redirect 覆盖链诊断见 2026-07-04 ResourceStreamer 行；RenderDoc/product capture、更广 product/perf sweep、editor project-sync refresh exact Cargo result 仍待后续切片；focused miss=0 已由 SH04 补跑关闭。 |
| 2026-07-04 | SH03-M2 redirect 覆盖链产品侧 ResourceStreamer 诊断 | 已完成聚焦验证 | `render_product_streamer_tests/readiness_diagnostics/shader_redirect.rs` 新增独立 child owner，锁定 ResourceStreamer 产品边界的两条 redirect 链路：resolved redirect 会递归 prepare include shader，并通过 `shader_module_include_sources(...)` 把 redirected include source 供给模板；missing redirect include 会在材质 readiness 中保留 `UnresolvedShaderReference` 与 shader fallback usage，但不阻塞 `ensure_material` 写入产品诊断。父 `readiness_diagnostics.rs` 保持 651 行，新 child 148 行。 | `rustfmt --edition 2021 --check --config skip_children=true zircon_runtime\src\graphics\scene\render_product_streamer_tests\readiness_diagnostics.rs zircon_runtime\src\graphics\scene\render_product_streamer_tests\readiness_diagnostics\shader_redirect.rs` 通过；`cargo test -p zircon_runtime --lib render_product_streamer_ --no-default-features --features target-server --locked --jobs 1 --target-dir E:\cargo-targets\zircon-shader-redirect-product-streamer --message-format short --color never -- --nocapture --test-threads=1` 的 Cargo wrapper 在 1204s 工具窗口内超时，不计通过，但随后留下可直接运行的 `E:\cargo-targets\zircon-shader-redirect-product-streamer\debug\deps\zircon_runtime-6bef7a696c15c9a5.exe`；直接运行该二进制 `shader_redirect --nocapture --test-threads=1` 通过 2/2、6399 filtered、2.46s。 | redirect 覆盖链产品诊断聚焦关闭；RenderDoc/product capture、更广 product/perf sweep、editor project-sync refresh exact Cargo result 仍待后续切片。 |

## 风险与回退

- 自实现解析器 vs naga_oil:维持自实现(仓库既定方向,避免 naga 版本锁耦合),但语义以 Composer 为对照基准;若后续需要 `#import 符号级导入`,以扩展词法层实现,不切换依赖。
- 词法级 `#include` 扫描误报(注释/字符串中的伪指令):解析器按行首指令处理并忽略行注释,测试覆盖;不为此引入完整 WGSL 预处理器。
- 模块级 properties(带属性的可复用模块)是已知的后续需求(UE Material Function 带参数的形态):当前禁止,避免布局归属复杂化;需求成熟后作为独立计划扩展,不在本计划夹带。

## Code Review 建议 (2026-07-30)

基于对 shader graphics 目录与 module import 契约的实际阅读。

### 与代码现状不符，需修订

- 「目标架构」写「注册表与解析器在 `graphics/shader/module_registry/`(由现 `include_registry` 原位演进,硬切换改名)」,但实际落点是单文件 `zircon_runtime/src/graphics/shader/template/module_registry.rs`(仍在 `template/` 下,不是独立的 `graphics/shader/module_registry/` 目录)。SH03-M1 状态行已写「`include_registry.rs` 已硬切换为 `module_registry.rs`」但沿用了 template 子路径。建议把目标架构的归属路径更正为 `graphics/shader/template/module_registry.rs`,与实际模块 mount 一致(front-matter 无此字段,勿动)。
- 「模块 id/依赖契约在 `core/framework/render/shader/`」的落点已实现为 `zircon_runtime/src/core/framework/render/shader/module_import.rs`(提供 `#include` 扫描/剥离与 `self::`/`zr_*` 分类,SH03-M1 状态行已确认)。目标架构段建议点名该具体文件,便于后续 owner 定位,而不是只写目录。
