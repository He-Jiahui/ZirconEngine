---
handoff_kind: fixed
status: fixed
created_at: 2026-07-11
summary_slug: source-cubemap-source-texel-test-api-drift
origin_plan: docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
fixing_plan: docs/plans/zircon_runtime/render/11-environment-lighting.md
origin_child_dir: docs/plans/zircon_editor/editor/10
fixing_child_dir: docs/plans/zircon_runtime/render/11
related_code:
  - zircon_runtime/src/core/framework/render/environment/source_cubemap/tests.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap.rs
plan_sources:
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
  - docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - cargo test -p zircon_runtime --locked
resolved_at: 2026-07-12
---


# Render 11：Source cubemap 测试 API 漂移失败交接

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md`
- 来源执行切片：Plan10 M1.1 manifest v2 与多资产根测试阶段
- 修复责任计划：`docs/plans/zircon_runtime/render/11-environment-lighting.md`
- 交接原因：Runtime 测试目标在执行 Plan10 manifest/scan/watcher 回归前编译全部 lib-test owner，当前最低错误位于 Render 11 source-cubemap 单测。

## 失败现象与复现证据

受管命令 `cargo test -p zircon_runtime --locked` 在编译 `zircon_runtime` lib-test 时返回 E0599 两处：

- `zircon_runtime/src/core/framework/render/environment/source_cubemap/tests.rs:278:17`
- `zircon_runtime/src/core/framework/render/environment/source_cubemap/tests.rs:282:17`

两处仍调用已不存在的 `SourceCubemapMipChain::source_texel`；当前生产类型只公开参数形状不同的 `source_texels`。这是 source-cubemap 测试与当前 Render 11 API 的漂移，不是 Plan10 manifest、多根注册或 watcher 行为失败。编译在生成 runtime lib-test 二进制前终止，因此 Plan10 runtime 测试未执行。

权威诊断：`D:\targets\zircon-engine\lanes\test-d1f1360952c942f49ffe9443ebd6b853\debug\.fingerprint\zircon_runtime-d204f127672c3c4*\output-test-lib-zircon_runtime`。

## 最低共享层根因

Render 11 的测试仍依赖已从当前 `SourceCubemapMipChain` 移除的单 texel helper，而生产 owner 已收敛为 `source_texels` 切片访问。最低失配位于 source-cubemap owner 与其同目录测试之间。

## 架构修复验收

- 由 Render 11 owner 按当前 source/source-mip 数据模型更新测试采样方式，或在确有生产需要时恢复单点采样 API 并给出公共合同；不得只为解锁其他计划添加兼容 shim。
- 保留测试原本对 source cubemap texel 的行为断言，不能删除、`cfg` 跳过或替换成恒真断言。
- 先复验对应 source-cubemap exact/lib-test 编译，再通知 Plan10 owner重跑 `cargo test -p zircon_runtime --locked`。

## 禁止临时方案

- 禁止恢复只供旧测试调用的兼容 alias、静默 fallback 或 call-site 特例。
- 禁止删除、跳过或弱化 source texel 行为断言来隐藏 API 漂移。

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| EL-M1/EC-M2 | SourceCubemapMipChain source texel 测试 API 对齐 | `未通过-待-render-owner-修复` | 2026-07-11 | Plan10 M1.1 受管 Runtime gate 在 `source_cubemap/tests.rs:278,282` 报 E0599×2；211/212 interface tests 已独立通过，但 runtime lib-test 未生成、Plan10 runtime 断言未执行。 |

## 修复结果与回传

- 根因：The source-cubemap test retained a removed single-texel method instead of indexing the canonical source_texels face-major layout.
- 架构修复：Kept SourceCubemapMipChain public surface on source_texels and source layout accessors; the co-owned test now derives a checked face/mip offset through source_cubemap_face_mip_offset without a compatibility shim.
- 验证：Managed Windows exact test source_cubemap_samples_equirect_uv_from_cube_face_direction passed 1/1 on the current lib-test binary.
- 回传：Editor 10 may rerun its zircon_runtime package gate; the Render 11 source-cubemap API drift no longer blocks lib-test compilation.
