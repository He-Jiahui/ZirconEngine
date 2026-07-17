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
- Render01 owner 完成消费者迁移后，Frameworks02 对原 30 个 E0616 文件做只读复扫：剩余 compiled-pipeline 非方法 `.graph` 访问 0、直接 `.enabled_features` 访问 0；9 个 `CompiledRenderPipelineParts` 引用均由 crate-internal canonical route 解析，原 E0308 已通过 `match &lifetime.desc` 收口。
- Render01 exact 64-path Rust manifest 以 forward-slash `path_key`、`[StringComparer]::Ordinal` 排序、`path=lowercase_sha256`、LF/no-final-LF 连续两次重算稳定为 `3744b95eb59e104dac132964a1bf3eeea366919e2ac6393051afbfa415b46e32`。此前 `bec06a53...` 是 culture sort 产物，已明确拒绝，不提供兼容。
- source-bound focused `plugin` 预约没有建立、Cargo 没有重跑：协调器在解析 `source_manifest` 前以 `Cargo compatibility field build_config is empty or invalid` 拒绝 64-path payload。最低共享层原因为 `CargoCompatibility::canonical()` 仍限制 `build_config <= 4096` 字符，而 64 个 SHA-256 值本身已超过该上限；该问题已通过 Session `frameworks02-m1-current-source-acceptance-r10-20260717` 的 `resolving_failure` reason 回传 Coordinator01。

## Current Blocker

- Render01 的 compiled-pipeline hard cut 已在静态层收口，并保持字段私有；当前阻塞已下沉到 Coordinator01 的 exact source-manifest 预约载荷契约，不再是 Render01 源码诊断未修复。
- Coordinator01 必须在不弱化逐文件 SHA-256、预约时校验和 consume/start 前复验的前提下，移除或提升与 64-path manifest 冲突的 4096 字符上限，并补充大清单回归测试；Frameworks02 不用无绑定预约、人工时间窗或部分清单冒充原子 current-source gate。
- 修复装载后，Frameworks02 只接受聚合指纹仍为 `3744b95eb59e104dac132964a1bf3eeea366919e2ac6393051afbfa415b46e32` 的 64/64 清单并原样重跑 focused `plugin`；随后继续 `descriptor`、`registration`、package compile 与插件工作区 gates。任何源码漂移都必须重新授权，不接受已拒绝的 `bec06a53...`。

## Review

- 独立只读复审：Critical 0 / Important 0 / Minor 0。确认 21 个 typed fields/freeze/is_frozen 集合一致、无 legacy/compat/shim、descriptor/module registration 单源，以及 exact integration 覆盖完整四阶段生命周期。
- 本记录当前为 `in_progress`；不把 exact 1/1 或静态门冒充 M3 全波次接受。
