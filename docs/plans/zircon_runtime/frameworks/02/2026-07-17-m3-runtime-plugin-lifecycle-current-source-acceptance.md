# Frameworks 02 M3 RuntimePlugin Lifecycle Current-Source Acceptance

Plan: docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md
Milestone: M3
Status: in_progress
Files: ["tools/tests/test_plugin_extension_registry_finalize_coverage.py", "docs/plans/zircon_runtime/frameworks/02/2026-07-17-m3-runtime-plugin-lifecycle-current-source-acceptance.md"]

> 本记录按当前共享源码验收 RuntimePlugin 生命周期并轨。它只记录已取得的 fresh evidence；在 focused、插件工作区、failure audit 与原生 milestone gates 全部关闭前，不把 M3 写成 completed。

## Scope Delivered

- `RuntimePluginDescriptor` 只内嵌一份内核 `ModuleDescriptor`；`RuntimePlugin` 的 lifecycle 访问直接投影该 descriptor，`RuntimePluginRegistrationReport::from_plugin` 只注册一份 module row。
- 当前第一方插件结构审计覆盖 28 个 RuntimePlugin descriptor roots：0 descriptor single-source violations、0 free-function registration sites、0 registration compatibility shim sites。
- `plugin_interface_imports` 加入后，`RuntimeExtensionRegistry` 已有 21 个 typed extension points；本轮把 finalize coverage guard 的显式基数从 20 同步到 21，并继续要求 typed fields、`freeze()` 与 `is_frozen()` 三集合完全相等。
- 不恢复旧 PluginLifecycle、兼容 alias/shim、第二套 module descriptor 或第二次 module registration。

## Fresh Testing Evidence

- Windows managed exact integration：reservation `86617287665b467e8fa2834ba65f8317`，job `4b4e5bd0421842f6ae7453c7f75cbd54`，run `cc978af1cc33401fb39b46d56ca261bc`；`cargo test -p zircon_runtime --test frameworks02_runtime_plugin_lifecycle --locked --jobs 1 --color never -- --test-threads=1`，1/1 passed，exit 0。覆盖单 module row 与 `build -> ready -> finish -> cleanup`。
- `python tools/tests/test_plugin_extension_registry_finalize_coverage.py`：RED 为 `20 != 21`；基数同步后 4/4 passed。
- `python -m unittest tools.tests.test_plugin_structure_audit_registration`：10/10 passed。
- `python tools/tests/test_runtime_plugin_descriptor_provided_interface_projection.py`：1/1 passed。
- `python tools/tests/test_audit_plugin_structure_report.py`：4/4 passed；`audit_plugin_structure.py --json` 报告 28 roots、0 单源违规、0 compat shim。
- focused `plugin` managed attempt：job `e4f2c4c6cb0e42f29f1674210b5bccb5` / run `fa98f9f4a3df40c4a4dfc358929dc404` 在执行测试前 exit 101；唯一错误为 foreign Render01 active change 的 E0425，不计作 plugin 测试失败。
- Render01 修复该 E0425 后的 focused `plugin` managed attempt：reservation `3749c1689b004d19a8468006f73405fa`，job `4f529b9a7e5b49eb8d76ee942f754d53`，run `dd942440ed9c423aaacfd791feb3eb3b`；仍在执行测试前 exit 101。fresh 编译诊断为 134 个 foreign Render01 hard-cut consumer errors / 37 个唯一文件：`CompiledRenderPipelineParts` 路径不可达 E0422 x9、私有 `CompiledRenderPipeline::graph` 旧直接访问 E0616 x124、fixture `TextureDesc` 借用类型漂移 E0308 x1；本次诊断中没有 `enabled_features` E0616。

## Current Blocker

- 最初 `RenderGraphResourceAccessKind` E0425 已由 Render01 owner 在其租约内修复；后续 run `dd942440ed9c423aaacfd791feb3eb3b` 证明更低层的 `compiled-pipeline-frame-derived-recomputation` hard-cut 仍未完成消费者迁移。
- Render01 owner 已重领 63-path expanded scope，并明确保持 `CompiledRenderPipeline` 字段私有：`CompiledRenderPipelineParts` 走 crate-internal canonical route，旧 `.graph` / `.enabled_features` 消费者改用新版只读访问面，不通过重新公开字段或 compatibility re-export/shim 收口。
- Frameworks02 已向 owner 回传 9 个 E0422 精确位置、30 个 E0616 文件及唯一 E0308；在 owner 回传 fresh source-manifest fingerprint 前不重跑 Cargo。fingerprint 到达后原样重跑 focused `plugin`，再继续 `descriptor`、`registration`、package compile 与插件工作区 gates。

## Review

- 独立只读复审：Critical 0 / Important 0 / Minor 0。确认 21 个 typed fields/freeze/is_frozen 集合一致、无 legacy/compat/shim、descriptor/module registration 单源，以及 exact integration 覆盖完整四阶段生命周期。
- 本记录当前为 `in_progress`；不把 exact 1/1 或静态门冒充 M3 全波次接受。
