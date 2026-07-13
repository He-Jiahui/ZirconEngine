---
handoff_kind: fixed
status: fixed
created_at: 2026-07-13
summary_slug: font-sdf-build-tool-root-surface-drift
origin_plan: docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
fixing_plan: docs/plans/zircon_runtime/text/05-sdf-msdf-pipeline.md
origin_child_dir: docs/plans/zircon_runtime/runtime/02
fixing_child_dir: docs/plans/zircon_runtime/text/05
related_code:
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/graphics/text/font_sdf_build_tool
  - zircon_runtime/src/bin/zircon_font_sdf_bake
  - zircon_runtime/src/graphics/mod.rs
  - zircon_runtime/src/graphics/text/mod.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_root_surface.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/core_spine_root_generated_boundary.py
tests:
  - python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py --json
  - rustc --edition 2021 --test zircon_runtime/src/tests/runtime_absorption/root_surface.rs
  - rustc --edition 2021 --test zircon_runtime/src/tests/runtime_absorption/core_spine_root_generated.rs
  - cargo test -p zircon_runtime --test runtime_text_sdf_offline_artifact --no-default-features --features font-sdf-build-tool --locked --jobs 1
  - cargo check -p zircon_runtime --bin zircon_font_sdf_bake --no-default-features --features font-sdf-build-tool --locked --jobs 1
resolved_at: 2026-07-14
---


# Text 05：font SDF build tool 引入未裁决 Runtime 根公开模块

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md`
- 来源执行切片：Runtime02 M2 service-registry 修复后的完整结构审计
- 修复责任计划：`docs/plans/zircon_runtime/text/05-sdf-msdf-pipeline.md`
- 交接原因：最低原因是 Text05 的 `font-sdf` target/CLI 为复用离线 bake 实现而新增 `pub mod font_sdf_build_tool`；模块内容与调用方均由 Text05 所有，但它改变了 Runtime02 已收敛的 crate-root public surface。

## 失败现象与复现证据

2026-07-13 当前源码执行：

```powershell
python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py --json
```

程序自然返回 0，但 `root_surface_audit` 报告：

- `public_module_count = 20`，Runtime02 当前已同步预期为 19；
- `unclassified_public_modules = ["font_sdf_build_tool"]`；
- `root_surface_migration_debt_count = 1`；
- `m1_gate_status = migration-debt-present`；
- `core_spine_root_generated_boundary.risks` 为 `Runtime root public module count changed without Runtime 02 audit sync.`。

`zircon_runtime/src/lib.rs` 的新 seat 受 `font-sdf-build-tool` feature 控制，但 cfg 并不会取消它作为公开根 API 的结构事实。CLI 的两个调用方位于 `src/bin/zircon_font_sdf_bake/{args,main}.rs`，均直接导入 `zircon_runtime::font_sdf_build_tool`。

## 最低共享层根因

Text05 把离线工具实现作为新的 crate-root public module 暴露，以便同包 binary crate 调用；该路径没有经过 Runtime02 root-surface decision table，也不属于既有 stable facade、namespace entry、runtime module entry 或 graphics/RHI deferred 分类。于是 Text05 的实现便利突破了 Runtime02 的根公开面合同。

## 架构修复验收

- 优先将 build-tool API 硬切到已有且语义匹配的 graphics/text 命名空间下的窄、feature-gated surface，或采用不新增 Runtime 根 seat 的同等结构；旧 `zircon_runtime::font_sdf_build_tool` 路径不得保留。
- 若 Text05 证明根 seat 是不可替代的正式 Runtime API，必须先由 Runtime02 明确裁决分类、文档和预期计数；禁止只把 19 改成 20。
- `runtime_root_surface` 必须恢复 `unclassified_public_module_count = 0`、`root_surface_migration_debt_count = 0`、`m1_gate_status = classified-and-clear`。
- 原始 aggregate audit 中 `core_spine_root_generated_boundary.risks` 必须清除该 root module count 风险。
- 重跑 root-surface、core-spine/root/generated、plan-status 与 Text05 font-sdf CLI/managed Cargo gates，证明 hard cutover 不破坏离线 bake。

## 禁止临时方案

- 禁止只提高 `EXPECTED_ROOT_PUBLIC_MODULE_COUNT`，或把未裁决模块标记为允许例外。
- 禁止保留 `pub use`、兼容模块、双路径或旧 `zircon_runtime::font_sdf_build_tool` shim。
- 禁止把 build-tool 实现复制到 binary 与 library 两处形成双重真相。
- 禁止削弱 root-surface/core-spine 审计以隐藏该失败。

## 修复结果与回传

- 根因：Text05 仅为让同包 CLI 复用离线字体 SDF 构建代码，新增了 feature-gated Runtime crate-root public module；该入口绕过 Runtime02 root-surface 分类，把公开模块数从 19 改成 20。
- 架构修复：实现硬切到 `graphics/text/font_sdf_build_tool`，仅在 `font-sdf-build-tool` feature 下公开 `zircon_runtime::graphics::text::font_sdf_build_tool`；CLI、测试和文档全部迁入新路径，删除 crate-root declaration，不保留 re-export、shim、双实现或旧导入路径。
- 验证：scoped rustfmt/diff check 通过，旧路径活动引用为 0；离线产物集成测试 2/2，`zircon_font_sdf_bake` Cargo check 返回 0；fresh default Runtime lib-test no-run 返回 0，`generated` 29/29，`core::` 705/705，root-surface 7/7，core-spine/root/generated 2/2，完整 structure-convention 1304/1304。聚合审计恢复 public modules 19/19、unclassified 0、migration debt 0、`classified-and-clear`、root risks 0。fresh `runtime_absorption` 从 1626/1637、11 failures 收敛为 1629/1637、8 failures，Text05 引入的 3 项 root-surface failure 全部消失。
- 回传：Runtime02 root public surface 与 generated mirror 门可以继续；剩余 8 项 `runtime_absorption` 失败归属 Runtime06/07/11/13/15，不重新打开 Text05 namespace ownership。
