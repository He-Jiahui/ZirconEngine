---
handoff_kind: fixed
status: fixed
created_at: 2026-07-15
summary_slug: irradiance-volume-shader-ide-validation-dependency
origin_plan: docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md
fixing_plan: docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
origin_child_dir: docs/plans/zircon_editor/editor/07
fixing_child_dir: docs/plans/zircon_runtime/render/18
related_code:
  - zircon_runtime/src/graphics/shader/wgsl/zr_lightmap.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_irradiance_volume.wgsl
  - zircon_runtime/src/graphics/shader/template/module_registry.rs
  - zircon_runtime/src/graphics/shader/ide_env_generation.rs
tests:
  - cargo test -p zircon_editor --lib --locked tests::editor_event::animation_runtime --jobs 1 -- --test-threads=1
resolved_at: 2026-07-15
---


# Render18: irradiance-volume Shader IDE validation dependency is incomplete

## 产出记录与时间

| 状态 | 记录日期 | 完成项目与当前门禁 |
|---|---|---|
| `OPEN / 待修复` | 2026-07-15 | Editor07 current-source 受管 job `c37d4e6f07f24e5d9424536d3c44b092` 已完成全量编译并通过 focused-document 2/2；随后 animation runtime 0/15 在共同的 `EditorAssetManager -> write_shader_ide_env_for_project` 前置阶段失败。Naga 报告 `zr_lightmap.wgsl` 调用未知函数 `zr_irradiance_volume_sample`，动画测试体尚未执行。 |

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md`
- 来源执行切片：Editor07 animation asset-index fixture failure return / current-source upward exact gate
- 修复责任计划：`docs/plans/zircon_runtime/render/18-advanced-lighting-features.md`
- 交接原因：`zr_irradiance_volume_sample`、`zr_irradiance_volume.wgsl`、lightmap priority chain 与 builtin module registration 都由 Render18 AF-M2 irradiance-volume 切片引入；Editor07 只是在产品资产同步阶段消费 Shader IDE 环境。

## 失败现象与复现证据

受管 Windows job `c37d4e6f07f24e5d9424536d3c44b092` 在当前源码完成 `zircon_editor` lib-test 编译后运行
`tests::editor_event::animation_runtime`，15 项全部在动画 graph/state-machine/sequence 逻辑之前失败。共同错误为：

`EditorAssetManager` 初始化刷新 Shader IDE 环境时，`zr_lightmap.wgsl` 第 121 行调用
`zr_irradiance_volume_sample(...)`，但该 stub 的验证源没有组合 `zr_irradiance_volume.wgsl`，Naga 返回
`unknown identifier: zr_irradiance_volume_sample`。完整输出位于
`E:/ZirconBuilds/editor07-failure-return-animation-runtime-20260715.out.log`。

## 最低共享层根因

Render18 已把 `zr_irradiance_volume.wgsl` 注册进 builtin module registry，也已让产品 forward/deferred
组合源码在 `zr_lightmap.wgsl` 之前拼接该模块；但 Shader IDE 的单 stub 验证依赖图仍只追加旧的 common builtin
集合。`zr_lightmap.wgsl` 没有一条可被 Shader IDE 依赖解析器识别的 irradiance-volume 依赖，因此产品组合顺序
与 IDE 单模块验证形成两个不一致真源。

## 架构修复验收

- 由 Render18 把 lightmap -> irradiance-volume 依赖声明到唯一 module/dependency graph，禁止仅在 Editor 初始化调用点拼接源码。
- 增加 focused Shader IDE regression，证明 `zr_lightmap.wgsl` stub 在 builtin dependency context 下可解析，并覆盖 `zr_irradiance_volume_sample`。
- 重跑 Render18 AF-M2 focused shader/product tests，确认 forward/deferred 组合顺序与绑定 35..37 不回退。
- 重跑 Editor07 `tests::editor_event::animation_runtime`、`tests::editor_event::runtime::animation_assets` 和 animation reflection action upward gate。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- Do not weaken tests or plan acceptance criteria to hide the failure.
- 禁止在 Editor07 测试或 `EditorAssetManager` 中跳过 Shader IDE 刷新；禁止为 Naga 注入空函数或仅调整测试顺序。

## 修复结果与回传

- 根因：Shader IDE standalone-stub validation expanded only direct includes, so lightmap and Standard PBR lost irradiance-volume and volumetric transitive dependencies that product shader assembly already resolved.
- 架构修复：Declared lightmap, PBR extras, Standard PBR, light-cookie, irradiance-volume, and volumetric dependencies in the builtin ShaderModuleRegistry and made Shader IDE stub validation traverse that dependency graph recursively with deterministic cycle and duplicate suppression.
- 验证：Current-source zircon_runtime test executable passed the all-builtin shader IDE stub generation/Naga validation test plus exact lightmap-to-irradiance, Standard-PBR transitive, PBR-extras-to-volumetric, and Standard-PBR dependency-order regressions (5/5). Editor upward job bd7478387049469c9a8dce29cdda7fa6 reached Runtime compilation and was bounded by unrelated active Runtime10 dynamic_api/session visibility errors, with no AF-M2 diagnostic.
- 回传：Returned the single-registry recursive Shader IDE dependency fix with focused current-source Naga proof; isolated coordinator validation remains the authoritative cross-worktree gate.
