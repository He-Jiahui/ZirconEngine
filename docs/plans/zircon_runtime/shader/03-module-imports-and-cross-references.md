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
| 2026-07-03 | SH03-M2 失效传播与插件注册口 | include 预热失效传播完成；插件注册口待补 | 模块 include 的 content_hash 已进入 template assembly 的 include content hash 列表，可参与现有磁盘 shader hash；引用资产依赖会通过 material/shader dependency set 基础链路进入加载。`zircon_shader_prewarm` asset-root 扫描现在保留 `.zshader` 的 kind/import_path/imports，在生成 variants 前解析同 root 下 include 模块依赖，并把模块内容 hash 混入引用者 request 的 `include_content_hashes` 与 `ShaderVariantKey.material_revision`；无 include 依赖的旁观者 shader revision 保持不变。 | `rustfmt --edition 2021 --check --config skip_children=true zircon_runtime/src/bin/zircon_shader_prewarm/manifest.rs zircon_runtime/src/bin/zircon_shader_prewarm/manifest/revision.rs zircon_runtime/src/bin/zircon_shader_prewarm/manifest/tests.rs` 通过；`cargo test -p zircon_runtime --bin zircon_shader_prewarm shader_prewarm_asset_root_manifest_tracks_imported_include_module_revisions --locked --jobs 1 --target-dir F:\cargo-targets\zircon-runtime-shader-sh03-readiness-0704b --message-format short --color never -- --nocapture --test-threads=1` 通过 1/1；`cargo test -p zircon_runtime --bin zircon_shader_prewarm shader_prewarm_asset_root_manifest --locked --jobs 1 --target-dir F:\cargo-targets\zircon-runtime-shader-sh03-readiness-0704b --message-format short --color never -- --nocapture --test-threads=1` 通过 10/10，只有既有 warning。 | 插件 manifest 模块注册 API、真实编辑器热重载传播和产品级 RenderDoc/product capture 仍待后续切片。 |
| 2026-07-04 | SH03-M2 redirect 来源诊断补口 | 已完成聚焦验证 | `ShaderImportReadiness` 新增只读 `source_diagnostic`，redirect 行会记录 declared import path 与 redirect locator；source-only import 保持 `None`，不改变 readiness ready/fail 判定、依赖计数、ResourceStreamer 供给、模板拼接或渲染行为。 | `rustfmt --edition 2021 --check --config skip_children=true zircon_runtime/src/asset/assets/shader/readiness.rs zircon_runtime/src/asset/tests/assets/shader_readiness.rs` 通过；`git diff --check` scoped 到同两文件通过，仅报告既有 LF/CRLF 提示；首次 `E:\cargo-targets\zircon-runtime-shader-sh03-readiness-0704` 运行在 `proc-macro2` build-script 链接阶段遇到 MSVC `link.exe` exit code 1318，未编译项目代码且不计结果；改用 `F:\cargo-targets\zircon-runtime-shader-sh03-readiness-0704b` 后 `cargo test -p zircon_runtime --lib shader_readiness_reports_import_rows_without_blocking_source_only_imports --locked --jobs 1 --target-dir F:\cargo-targets\zircon-runtime-shader-sh03-readiness-0704b --message-format short --color never -- --nocapture --test-threads=1` 通过 1/1，只有既有 warning。 | 插件 manifest 模块注册、模块热改精确失效和旁观者不失效测试仍待后续 SH03-M2 产品化切片。 |

## 风险与回退

- 自实现解析器 vs naga_oil:维持自实现(仓库既定方向,避免 naga 版本锁耦合),但语义以 Composer 为对照基准;若后续需要 `#import 符号级导入`,以扩展词法层实现,不切换依赖。
- 词法级 `#include` 扫描误报(注释/字符串中的伪指令):解析器按行首指令处理并忽略行注释,测试覆盖;不为此引入完整 WGSL 预处理器。
- 模块级 properties(带属性的可复用模块)是已知的后续需求(UE Material Function 带参数的形态):当前禁止,避免布局归属复杂化;需求成熟后作为独立计划扩展,不在本计划夹带。
