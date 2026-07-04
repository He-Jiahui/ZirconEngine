# 计划 SH-05:shader 编写 DX——import_path 推导、IDE 可识别性与诊断回映

## 目标

1. `import_path` 默认由资产路径**确定性推导**,显式声明降级为覆写位:开发者在常规目录结构下零手写模块 id,include 路径"尽可能简单"。
2. 引擎生成 **IDE 解析环境**单一 artifact(模块物化 stub 树 + `module_map.json` + 拼接预览),使 `#include <logical::path>` 与生成符号(`zr_mat_*`、`ZrSurfaceInput` 等)对通用 WGSL 工具可见、可跳转;IDE/编辑器只消费该产物,不各自重建解析。
3. SH-02 生成属性模块登记模块 id `self::material`:用户 WGSL 可选写一行 `#include <self::material>` 作为 IDE 锚点,拼接器按模块 id 去重,写与不写产物逐字节一致。
4. 拼接产物 → 用户源的**行号回映**:naga 验证/编译诊断报(模块 id 或用户文件,局部行号),不再暴露拼接后行号。

非目标:不自研 WGSL 语言服务;不为特定 IDE 扩展迁就引擎语法;编辑器内置 shader 编辑面板归 editor 计划集,本计划只保证 artifact 完备。

## 现状与差距

- `import_path` 是自由字符串(SH-03 定稿其为模块身份),与文件位置无关联:开发者要为每个模块起名、记名,引用时无处对照——与"尽可能简单"的诉求相反。
- `#include <zircon::noise>` 是逻辑 id,磁盘上无对应物理文件,任何 IDE(VSCode + wgsl-analyzer 等)都无法跳转、无法索引其符号。
- SH-02 生成属性模块在拼接期注入,用户 `.wgsl` 源文件中不存在 `zr_mat_*`/`zr_tex_*`/`ZrMaterialProperties` 的任何声明;计划 08 模板提供的 `ZrSurfaceInput`/`ZrSurfaceOutput` 同样不可见——IDE 对用户 shader 全文件报未定义符号,语法检查形同虚设。
- naga 验证错误行号基于拼接后全文(生成模块 + imports 展开 + 模板段在前),与用户源行号对不上,报错定位靠人工减行号。

## 参考代码

| 文件 | 应重点阅读 |
|------|-----------|
| `dev/bevy/crates/bevy_pbr/src/render/pbr_bindings.wgsl` 等 | `#define_import_path` 把模块身份写进物理文件,wgsl-analyzer 生态因此可解析——本计划身份留在 zshader,由 stub 树补物理位,对照其"物理文件即模块"的可识别性来源 |
| `dev/Graphics/.../core/ShaderLibrary/Common.hlsl` 与 URP `ShaderLibrary/` | 物理相对路径 include 天然 IDE 可解析——stub 树目录形态(`::` → `/`)的基准 |
| `dev/bevy/crates/bevy_shader/src/shader_cache.rs` | 组合期错误经模块名回报(`ComposerError` 带模块上下文)——诊断回映的语义同类 |
| 本仓库 `graphics/shader/template/{assemble,include_registry}.rs` | 拼接顺序与注入点:段表(`AssembledSegment`)在此记录;SH-03 的 `module_registry` 演进后同挂点 |
| 本仓库 `zircon_runtime/src/bin/zircon_shader_prewarm/` | 资产扫描型 CLI 的结构样板(args/run/manifest 分层)——`zircon_shader_ide_env` 同构新建 |
| `docs/plans/zircon_runtime/render/08-material-shader-permutation.md` 工程落地细化 | 拼接器细分顺序与磁盘缓存布局;段表与预览文件的登记目标 |

## 目标架构

归属:import_path 推导规则与 `module_map` 契约类型在 `core/framework/render/shader/`;推导执行与校验在 `asset/importer/ingest/`;段表与回映在 `graphics/shader/template/`;IDE 环境生成器为 `zircon_runtime/src/bin/zircon_shader_ide_env/`(与 prewarm 同构)。

### import_path 默认推导(简单性)

- 推导规则:资产根下 `shaders/` 起,目录段 `/` → `::`,文件名去扩展;首段为项目命名空间(项目配置声明,缺省取项目名 snake_case)。例:`assets/shaders/cloth/common.zshader` → `myproj::cloth::common`。
- 文件名与末级目录同名时折叠一段(`shaders/noise/noise.zshader` → `myproj::noise`,对齐现有 `default_pbr/default_pbr.zshader` 目录习惯)。
- 显式 `import_path` 仅在需要偏离推导值时声明;与推导值一致时 importer 报冗余 warning(可自动修复移除)。保留段(`zircon::`/`zr_`)与冲突校验沿 SH-03 不变。
- 推导只作用于 `kind = include`(必填身份);其余 kind 的可选 `import_path` 同规则推导。

### IDE 解析环境(生成物,单一来源)

生成器 `zircon_shader_ide_env` 扫描项目资产(复用 prewarm 的资产扫描面)+ 内建注册表 + 插件注册模块,输出到 `<project>/.zircon-cache/shader_ide/`(git-ignore,schema 版本目录同计划 08 磁盘缓存纪律):

```
shader_ide/v1/
  modules/zircon/noise.wgsl            ← 每个模块物化为物理文件(:: → /),头部注释标 id 与源资产 URI
  modules/myproj/cloth/common.wgsl
  generated/hero_cloth.material.wgsl   ← SH-02 生成属性模块 + 模板公共类型(ZrSurfaceInput 等)stub
  preview/hero_cloth.default.wgsl      ← 可选:默认变体拼接全文(--variants 扩展)
  module_map.json                      ← import_path → { stub 路径, 源资产 URI, 内容哈希 }
```

- **基线可用性(零扩展)**:stub 树本身可读、可搜索;在 stub/preview 文件上打开 wgsl-analyzer 即获得完整符号解析;preview 回答"最终 WGSL 长什么样"。
- **进阶消费**:编辑器与(可选的)官方 VSCode 薄扩展消费 `module_map.json`,提供 `#include <...>` 的 document-link 跳转与 `zr_*` 符号跳到 stub;类型级分析仍交给既有 WGSL 工具,不自研语言服务。扩展落点归 tooling/editor 计划集,本计划只定稿 `module_map.json` schema 并保证其稳定。
- 陈旧控制:`module_map.json` 携带各模块内容哈希;编辑器在资产导入完成后自动增量刷新;CLI 幂等,可挂 build 钩子。

### 生成符号锚点:`self::material`

- SH-02 生成属性模块在 `module_registry` 登记为**每 shader 局部**的模块 id `self::material`(解析上下文 = 当前根 shader);`self::` 为保留命名空间,不可被资产/插件占用,不计入 `imports` 声明义务。
- 用户 WGSL 可写一行 `#include <self::material>`:IDE 经 module_map 解析到 `generated/<shader>.material.wgsl` 获得全部生成符号;拼接器按模块 id 去重——自动注入与显式 include 同 id,注入恰一次,**写与不写产物逐字节一致**。
- 该行是纯 IDE 锚点,非语义依赖;文档与项目模板默认带上这一行(新建 shader 脚手架生成)。

### 诊断行号回映

- `assemble.rs` 拼接时记录段表 `Vec<AssembledSegment { module_id, start_line, line_count }>`(覆盖模板段、生成模块、imports 展开、用户源);naga 验证与编译错误的行号经段表回映为 `(module_id | 用户文件路径, 局部行号)`,进 import 期诊断与帧诊断(`VariantMissReport` 附带编译失败详情时同口径)。
- 段表随 `ResolvedVariant` 在调试构建保留;`preview/` 文件行号与段表一致,可作人工对照。
- 拼接顺序细分与段表结构登记进计划 08"工程落地细化"(与 SH-02 注入位置同一条目扩展)。

## 里程碑

前置:SH-02(生成模块存在)、SH-03(module_registry 与依赖解析存在);可与 SH-04 并行。

### SH05-M1 import_path 推导与 `self::material` 锚点

实施切片:
1. 推导规则实现与 importer 接线(冗余 warning、保留段/冲突校验复用 SH-03);项目命名空间来源接项目配置。
2. `self::` 保留命名空间与 `self::material` 局部模块注册;`assemble.rs`/`module_registry` 按 id 去重覆盖显式 include 情形;imports 一致性校验豁免 `self::`。

测试阶段:
- `cargo check -p zircon_runtime --lib --locked`;`cargo test -p zircon_runtime shader_module --locked`
- 验收证据:推导矩阵单测(常规/同名折叠/覆写/冗余/保留段);显式 `#include <self::material>` 与不写的拼接产物逐字节相等断言。

当前状态(2026-07-03):M1 验收切片已落地。`core/framework/render/shader/module_import.rs`
提供 `derive_shader_import_path(...)`、项目名 snake_case 命名空间推导、`shaders/`
起点裁剪、同名目录/文件折叠、`self`/`zircon`/`zr_*` 保留命名空间拒绝;compound
`.zshader` importer 会在 surface/include 缺省 `import_path` 时使用推导值,显式等于
推导值时写入冗余 warning,显式 `self::...` 或 `zr_*::...` 覆写时拒绝。项目扫描把
项目名作为内部 shader import 设置写入 config hash,并在同一扫描批次报告重复
`import_path` 冲突。`self::material` 已改为生成材质模块的局部 id,模板装配中显式
include 与自动注入逐字节等价。

本轮验证:
- `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh05-m1 --message-format short --color never`:通过,仅既有 warning。
- `cargo test -p zircon_runtime --lib export_runtime_shader_material_sphere_png --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh05-m1 -- --ignored --nocapture`:通过 1/1,导出 1024x1024 材质球截图 `docs/tests/runtime/shader/runtime_shader_material_sphere_offscreen_20260703.png`,SHA256 `927A652BFB6486145C9F6CDBD2E5EE49ED132DAA5290013422FD0C2E9769B794`,采样可见像素 137956、唯一颜色 1710、亮度范围 48.66;同名 target/cargo-targets 扫描为 0。
- `cargo test -p zircon_runtime --lib shader_module --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh05-shader-module --message-format short --color never -- --nocapture --test-threads=1`:通过 6/6,覆盖 module import 指令解析/剥离/builtin token 分类和 shader module registry 传递依赖/去重/环诊断/include directive 剥离;前序 Windows 编译/链接拥堵超时记录已被本次完整结果取代。

M1 后续项:IDE 侧脚手架提示随编辑器刷新钩子收敛;缓存恢复路径上的重复
`import_path` 扫描仍需专门补测。focused `shader_module` 过滤测试已补跑通过,不再作为
M1 待办项;本轮 import_path/self::material/材质球截图验收结论保持通过。

### SH05-M2 `zircon_shader_ide_env` 生成器与 module_map

实施切片:
1. `module_map.json` 契约类型(schema 版本、哈希)与生成器 bin(扫描、stub 物化、generated stub;`--variants` 预览可选);`.gitignore` 与文档。
2. 编辑器导入后增量刷新钩子(editor 侧仅挂调用,面板归 editor 计划集);`zircon_build.py`/staged 流程不打包 `shader_ide/`。

测试阶段:
- `cargo test -p zircon_runtime shader_ide --locked`(bin 内 `#[cfg(test)]` 按 prewarm 惯例)
- 验收证据:map 与 stub 树一致性(每 id 一文件、哈希匹配);模块改一字节 → 仅该 stub 与 map 条目变化(幂等/增量);生成 stub 全部过 naga。

当前状态(2026-07-03):M2 CLI 主路径、共享生成器、默认 preview 产物与编辑器导入后刷新钩子已落地。`core/framework/render/shader/ide_env.rs`
定义 `ShaderIdeModuleMap`、`ShaderIdeModuleMapEntry`、schema 版本、默认输出目录
`.zircon-cache/shader_ide/v1`、module stub 路径规则、per-shader
`generated/<shader>.material.wgsl` 路径规则、`preview/<shader>.default.wgsl`
路径规则和 `preview/<shader>.default.segments.json` 段表路径规则。`graphics/shader/mod.rs` 只公开
`builtin_shader_ide_module_sources()` 只读快照,避免把可变 `module_registry` 或旧
include registry path 暴露成公共 API。`graphics/shader/ide_env_generation.rs`
现在是 CLI 与 editor 共享的生成 owner:从 Ready shader artifact 写出内建模块 stub、
资产 `import_path` stub、`self::material` generated material stub 和 `module_map.json`;
默认输出在项目 `.zircon-cache/shader_ide/v1`,也可用 `--out-dir` 覆盖,`--pretty`
仅影响 CLI report。`zircon_shader_ide_env` bin 的 `run.rs` 保持为参数解析、项目扫描和 report
序列化薄壳;`DefaultEditorAssetManager::sync_from_project(...)` 在导入后扫描到 Ready shader
时调用同一生成函数刷新 `.zircon-cache/shader_ide/v1`,editor 面板不重建解析环境。`--variants` 保留为默认 preview 快捷入口,`--variant <pass[:options=bits]>`
可显式生成非默认组合 preview,例如 `gbuffer:options=0x1` 会写出
`preview/<shader>.gbuffer_options_0x00000001.wgsl` 与对应 `.segments.json`;
共享 DTO `ShaderIdePreviewVariant` 统一 CLI/editor/library 调用边界,生成器拒绝空名或同名变体,避免同一 preview 路径被重复写入。默认编辑器刷新仍只请求
static mesh + Forward pass + 全 option bits 关的默认变体;`graphics/shader/ide_preview.rs`
复用现有 template assembler,按请求的 pass 与 material option bits 收集 shader imports 的 include module 源,并把内部
`ShaderAssemblySegment` 转成 IDE 可消费的 `ShaderIdePreviewSegment`。生成器输出已改为
content-aware 写入:不再清空整个输出目录,只在内容变化时重写目标文件,并在
`modules/`、`generated/`、`preview/` 三个受管目录内清理过期产物;CLI report 同步记录
managed/written/removed-stale 文件数,用于验证模块一字节增量 diff 门禁。

2026-07-03 19:02 跟进:`graphics/shader/ide_validation.rs` 新增当前 DX 专用 Naga
WGSL 检查入口,`zircon_shader_ide_env` 在写入前对每个 stub 做 Naga parse,对
`--variants` 生成的默认 preview 做完整 Naga validate,并在 report 中输出
`naga_parsed_stub_count` 与 `naga_validated_preview_count`。单个 stub 可能引用其它
模块的类型/函数,因此本切片不把 standalone semantic validation 伪装成“全量成功”;
语义验证落在已拼接 preview 与后续非默认组合 preview 上。

2026-07-03 跟进:前序 `shader_ide_env` lib-test 超时和 editor refresh hook typecheck
阻塞已经由最低层修复与复跑关闭。SDF render folder-backed tests 的导入漂移已收敛到
`sdf_render/tests/mod.rs`,父 `sdf_render.rs` 不再替 child tests 承接 test-only import。
IDE stub 校验新增 validation-only 上下文:公共 builtins、默认 feature defines、
`self::material` generated stub 和当前 module stub 一起送入 Naga parse,但写出的 stub 文件
仍保持原本单文件形态。WGSL contract 侧把 `ZrSurfaceInput` alias、`zr_surface_default(...)`、
`ZrDeferredGBufferOutput` 和 deferred material flag encode helper 收敛进
`zr_surface_types.wgsl`,`zr_template_deferred_gbuffer.wgsl` 不再重复声明共享输出结构。

2026-07-04 跟进:非默认 preview 批量矩阵回归已补入
`shader_ide_env_batches_preview_matrix_for_all_surface_shaders`:测试构造两个 surface shader,
显式请求 default、GBuffer option bits、DepthPrepass、Shadow、Velocity 与
TAA reactive mask 组合,检查每个 surface × variant 的 preview 与 segments 路径,并在缩回
default-only 后检查 stale preview 清理。该切片目前只关闭测试代码编译门;lib-test
执行仍受 Windows 链接阶段超时影响,未计为通过。

M2 验证:
- `cargo check -p zircon_runtime --bin zircon_shader_ide_env --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh05-m2 --message-format short --color never`:通过,仅既有 warning。
- `cargo test -p zircon_runtime --bin zircon_shader_ide_env shader_ide_env --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh05-m2 --message-format short --color never -- --nocapture --test-threads=1`:通过 2/2,覆盖 module_map/stub/generated material 输出与幂等 report 序列化。
- `cargo check -p zircon_runtime --bin zircon_shader_ide_env --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh05-m4-preview --message-format short --color never`:通过,仅既有 warning;首次冷构建后台超时后复跑拿到完整通过结果。
- `cargo test -p zircon_runtime --bin zircon_shader_ide_env --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh05-m4-preview --message-format short --color never -- --nocapture --test-threads=1`:通过 5/5,覆盖 `--variants` 参数解析、默认 preview WGSL、`.segments.json` 段表、module_map/stub/generated material 输出、report 序列化和模块一字节增量 diff 只重写变化 stub 与 map。
- `cargo check -p zircon_runtime --bin zircon_shader_ide_env --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh05-m4-preview --message-format short --color never`(2026-07-03 19:02 Naga stub parse / preview validate gate):通过,仅既有 warning。随后聚焦 bin test 两次被外部 runtime/editor/plugin cargo/rustc 构建竞争拖到 304s 超时,未产生新测试结果,不计通过。
- `cargo test -p zircon_runtime --lib shader_ide_preview_paths_are_scoped_by_source_uri_and_variant --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh05-m4-preview --message-format short --color never -- --nocapture --test-threads=1`:未计通过;lib-test 编译被无关 `graphics/scene/scene_renderer/ui/text/sdf_fallback.rs` 的缺失 `mod tests;` 文件挡住。
- `rustfmt --edition 2021 --check --config skip_children=true` 覆盖 `graphics/shader/ide_env_generation.rs`、其 child tests、`zircon_shader_ide_env/run.rs`、graphics 导出、editor import refresh hook 与 ResourceStreamer accessor 导入修复:通过。
- `cargo check -p zircon_runtime --bin zircon_shader_ide_env --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh05-m4-preview --message-format short --color never`(2026-07-03 20:16 共享生成器/CLI 薄壳复验):通过,仅既有 warning。
- `cargo test -p zircon_runtime --bin zircon_shader_ide_env --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh05-m4-preview --message-format short --color never -- --nocapture --test-threads=1`(2026-07-03 20:16 生成器移入 lib 后):通过 1/1,只覆盖 CLI `--variants` 参数解析;原 generator 行为测试已迁入 `graphics/shader/ide_env_generation/tests.rs`,不再计入 bin 测试数量。
- `cargo check -p zircon_editor --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh05-editor-refresh --message-format short --color never`:通过,仅既有 warning;先在 SDF test-owner import 修复后关闭 editor refresh hook typecheck,再在 SH05 WGSL contract 收敛后复跑仍通过。
- `cargo test -p zircon_runtime --lib shader_ide_env --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh05-m4-preview --message-format short --color never -- --nocapture --test-threads=1`:通过 6/6,覆盖共享 library generator、module_map/stub/generated material、默认 preview/segment、增量 diff、stub Naga parse validation context 和 invalid-stub 诊断;前序 908s 超时不再代表当前状态。
- `cargo test -p zircon_runtime --lib shader_template_ --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh05-m4-preview --message-format short --color never -- --nocapture --test-threads=1`:通过 20/20,复验 `ZrDeferredGBufferOutput`/deferred flag helper 从 GBuffer template 移入 `zr_surface_types.wgsl` 后,template 拼接、段表和诊断回映仍稳定。
- `rustfmt --edition 2021 --check --config skip_children=true` 覆盖 SH05 shared generator、child tests、CLI 薄壳、editor refresh hook、SDF render/test owner 导入修复和相关 shader/template Rust 文件:通过。整包 `cargo fmt -p zircon_runtime --check` 仍被无关既有 `graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs` 格式漂移挡住,不作为本切片通过证据。
- `rustfmt --edition 2021 --check --config skip_children=true` 覆盖 `ShaderIdePreviewVariant` DTO、IDE preview generator/child tests、CLI args/run、graphics 导出与 editor refresh hook:通过。
- `cargo check -p zircon_runtime --bin zircon_shader_ide_env --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh05-nondefault-preview --message-format short --color never`:通过,仅既有 warning。
- `cargo check -p zircon_editor --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh05-editor-refresh --message-format short --color never`:通过,仅既有 warning。
- `cargo test -p zircon_editor --lib sync_from_project_refreshes_shader_ide_environment_after_import --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh05-editor-refresh --message-format short --color never -- --nocapture --test-threads=1`(2026-07-04 编辑器导入后刷新精确回归):源码测试已覆盖 `module_map.json`、默认 preview WGSL 与 `.segments.json` 写出;本次 Windows Cargo 运行在 904s 工具窗口内超时且未留下可直接运行的 editor test binary,未产生可计数结果。
- `cargo test -p zircon_runtime --lib shader_ide_env_writes_non_default_preview_variants_with_option_bits --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh05-nondefault-preview --message-format short --color never -- --nocapture --test-threads=1`:通过 1/1,覆盖 GBuffer preview、`material_option_bits=1`、`ZR_OPT_*` true define、`.segments.json` 变体名/路径与 Naga validate。
- `cargo test -p zircon_runtime --lib shader_ide_env_rejects_duplicate_preview_variant_names --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh05-nondefault-preview --message-format short --color never -- --nocapture --test-threads=1`:通过 1/1,覆盖同名 preview variant 拒绝。
- `cargo test -p zircon_runtime --bin zircon_shader_ide_env parse_accepts_non_default_preview_variant_specs --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh05-nondefault-preview --message-format short --color never -- --nocapture --test-threads=1`:通过 1/1,覆盖 `--variant gbuffer:options=0x1` 与非默认 pass 解析。
- `cargo test -p zircon_runtime --bin zircon_shader_ide_env parse_accepts_variants_flag --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh05-nondefault-preview --message-format short --color never -- --nocapture --test-threads=1`:通过 1/1,覆盖默认 `--variants` 快捷入口。
- 截图复核:`docs/tests/runtime/shader/runtime_shader_material_sphere_offscreen_20260703.png`(1024x1024) 与 `docs/tests/runtime/shader/runtime_shader_material_vampire_offscreen_20260703.png`(1280x720) 均在 `docs/tests/runtime/shader` 下,人工检查为非空渲染帧;材质球有完整轮廓与光照梯度,示例场景有可识别材质/物体输出。
- `rustfmt --edition 2021 --config skip_children=true zircon_runtime/src/graphics/shader/ide_env_generation/tests.rs`:通过,覆盖 preview matrix 测试与 fixture helper 重排。
- `cargo check -p zircon_runtime --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh05-preview-matrix-check --message-format short --color never`:通过,仅既有 warning;覆盖新增 preview matrix 测试代码编译。
- `cargo test -p zircon_runtime --lib shader_ide_env_batches_preview_matrix_for_all_surface_shaders --locked --jobs 1 --target-dir F:\cargo-targets\zircon-runtime-shader-sh03-readiness-0704b --message-format short --color never -- --nocapture --test-threads=1`:通过 1/1,覆盖两个 surface shader × default/GBuffer options=1/DepthPrepass/Shadow/Velocity/TAA preview 变体、per-shader generated `self::material` scope、Naga validate 计数与 stale preview 清理。首轮复跑先暴露两个真实缺口并已修复:`shader_ide_stub_validation_dependencies(...)` 对同名 generated material include 按 `scope_uri` 收敛,避免多 shader 项目把另一个 shader 的 `self::material` 追加进验证上下文;`zr_template_taa_reactive_mask.wgsl::fs_taa_reactive_material_mask` 改为从 `zr_material_surface(input).custom0.x` 读取 authored mask,不再要求自定义 surface preview 存在旧 `standard_material_properties`。
- `cargo test -p zircon_runtime --lib taa_reactive_mask --locked --jobs 1 --target-dir F:\cargo-targets\zircon-runtime-shader-sh03-readiness-0704b --message-format short --color never -- --nocapture --test-threads=1`:通过 13/13,复验 TAA template source、WGPU pipeline 创建、processor、diagnostics 与 material strength 相关守卫。

M2 已关闭项:CLI 主路径、library generator 行为测试、map/stub 一致性、默认 preview
拼接文件、段表 JSON、模块一字节增量 diff 门禁、全 stub Naga parse 写入前 gate、默认
preview Naga validate、编辑器导入后增量刷新钩子接线与 editor lib typecheck、focused
`shader_module` Cargo filter 6/6、显式非默认组合 preview 扩展及其聚焦语义 Naga 门禁、两个 surface shader 的产品级批量 preview matrix 执行门禁。M2 未完成项:RenderDoc/product capture 与更广 product/perf sweep 仍未关闭; product material-pass 二次启动 miss=0 已由 SH04 聚焦用例补跑关闭。编辑器导入后刷新精确 Cargo 回归已有源码覆盖,但本轮 Windows 构建窗口尚未给出可计数结果。
2026-07-04 已补入并执行通过产品级批量矩阵回归,同时补齐多 shader generated `self::material` scope 与 TAA material-mask 新 ABI 预览验证缺口。

### SH05-M3 段表与诊断行号回映

实施切片:
1. `AssembledSegment` 段表记录与错误行号回映;import 期诊断与帧诊断统一走回映后坐标。
2. preview 文件与段表行号一致性;计划 08 工程落地细化登记。

测试阶段:
- `cargo test -p zircon_runtime shader --locked`
- 验收证据:构造用户源第 N 行错误与 include 模块内错误各一例,诊断分别落 `(用户文件, N)` 与 `(模块 id, 局部行)`;模板段错误报模板 id 而非用户文件。

当前状态(2026-07-03):M3 诊断回映主路径已落地。`graphics/shader/template/assemble.rs`
新增 `ShaderAssemblySegment`、`ShaderAssemblySegmentKind`、`ShaderAssemblyBuilder`
和 `shader_assembly_source_location_for_line(...)`;Forward material、Deferred GBuffer
和 TAA reactive mask 模板装配都会返回段表,并区分 defines、include、generated
material、用户 surface 与 pass template 段。`validation.rs` 新增带段表的验证入口,
从 naga `SourceLocation` 取 assembled line/column 并追加
`Zircon shader source: <module_id>:<local_line>:<column>` 回映行。运行时 mesh
pipeline cache 会从 `ResourceStreamer` 读取 surface shader `import_path`,作为 Forward、
GBuffer 与 TAA pass 的用户源 module id;没有显式 import_path 时回退 `self::surface`。

本轮同时按结构规范修复两个预算风险:`graphics/shader/template/tests.rs` 的 surface
module regressions 已下沉到 `graphics/shader/template/tests/surface_modules.rs`;`mesh_pipeline_cache/ensure_pipeline.rs`
的 owner tests 已下沉到 `mesh_pipeline_cache/ensure_pipeline/tests.rs`,结构守卫同步锁定
child owner 与父文件 mount,避免后续 shader/material pass 继续堆进 umbrella 文件。

M3 验证:
- `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh05-m3 --message-format short --color never`:通过,仅既有 warning。
- `cargo test -p zircon_runtime --lib shader_template_ --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh05-m3 --message-format short --color never -- --nocapture --test-threads=1`:通过 20/20,覆盖段表记录、naga parse error 回映、template surface module child tests 与结构预算守卫。

M3 后续项:默认 preview 文件已把段表行号导出到 `.segments.json`;帧诊断里更完整的
编译失败上下文仍需在实际 pipeline failure report 中继续接入。本轮只声明模板
装配/验证/remap 主路径与 IDE 默认 preview 段表完成。

## 测试与验收清单

- `render_shader_import_path_derivation_*`:推导/折叠/覆写/冗余 warning/保留段拒绝。
- `render_shader_self_material_anchor_*`:锚点去重与产物逐字节等价。
- `shader_ide_env_module_map_*`:map–stub 一致性、增量幂等、陈旧哈希检测、stub 过 naga。
- `render_shader_diagnostic_remap_*`:三来源(用户源/资产模块/模板段)行号回映精确性。

## 风险与回退

- 外部 IDE 工具演进(wgsl-analyzer 对自定义预处理/`#include` 语义的支持度不可控):引擎侧只承诺 `module_map.json` 与 stub 树的稳定 schema;扩展层薄且可替换;**不**因工具限制改动 SH-03 已定稿的 `#include` 语法。
- stub 陈旧误导(用户读到旧 stub):map 哈希 + 编辑器导入后自动刷新兜底;CLI 文档明示"陈旧以引擎诊断为准";不做文件系统 watcher 双轨。
- preview 变体爆炸:默认只产默认变体(几何源 static、全 option 关、Forward pass),更多组合显式 `--variants`,不全量枚举。
- `self::material` 心智负担(一行"可写可不写"的指令):脚手架默认生成 + importer 对"缺锚点且文件内引用了 `zr_mat_*`"给一次性提示(hint 级),不强制。
- 项目命名空间来源(项目配置字段尚未定稿):M1 先接"项目名 snake_case"缺省,配置字段落点随项目 manifest 计划定稿后收敛,不阻塞本计划。
