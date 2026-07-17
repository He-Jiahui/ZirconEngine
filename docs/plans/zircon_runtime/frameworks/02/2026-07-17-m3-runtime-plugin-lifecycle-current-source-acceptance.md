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
- Render01 owner 完成消费者迁移后，Frameworks02 对原 30 个 E0616 文件做只读复扫：剩余 compiled-pipeline 非方法 `.graph` 访问 0、直接 `.enabled_features` 访问 0；9 个 `CompiledRenderPipelineParts` 引用均由 crate-internal canonical route 解析，原 E0308 已通过 `match &lifetime.desc` 收口。最终 frame-scan hard cut 又删除 `build_frame_submission_context` 对 `capability_requirements` 的逐帧扫描，将 ScreenSpaceAntiAlias capability 冻结在 runtime metadata descriptor；该 owner 聚合应精确验收 5 tests，但不能用全局 `plugin` 子串作为过滤器。
- Render01 first-class manifest 必须合并显式 Rust 文件租约与 owned Rust 目录租约的递归展开结果，再以 forward-slash `path_key`、`[StringComparer]::Ordinal` 排序、`path=lowercase_sha256`、LF/no-final-LF 聚合。最终 frame-scan 修复后的 68/68 路径原子重算命中 `4322914a3cb0f9709987f9745d1d7cba5b327fc3a0b378fe531e6e29d9213368`；目录租约继续补入 `resource_write_index.rs`、`runtime_feature_flags.rs`、`runtime_metadata.rs`、`tests.rs`。此前 `bec06a53...`、漏展开目录的 `3744b95e...` 与修复前的 `270fbc0b...` 均已明确拒绝，不提供兼容。
- Coordinator01 已把 `source_manifest` 提升为外层一等 compatibility payload，不再塞入受 4096 字符限制的 `build_config`；新 daemon instance `9405f5f59b464360958ec1f843380deb` / schema 47 在 reserve 与 consume/start 两个边界都对 68 个文件执行逐文件 SHA-256 复验。原 reservation `4a077dc751ed4b8a8f864affd22bc935` 因 owner session 心跳过期被合法标记为 `expired`，未 consume、未启动；恢复 session 后重新原子计算仍为 68/68、`4322914a3cb0f9709987f9745d1d7cba5b327fc3a0b378fe531e6e29d9213368`。
- Windows managed current-source 宽波：reservation `3fe31d7d89e54b2ab397a6bc2a6762e9`，job `42d5707f448d4589ae3ac390c0a19a1c`，run `c91570084d00499889ad5315db5a5d77`；`cargo test -p zircon_runtime --lib --locked --jobs 1 --color never plugin -- --test-threads=1` 完成冷编译，原 134 个 Render01 E0422/E0616/E0308 全部消失。但 Rust libtest 的 `plugin` 是全局子串过滤，实际执行 `820 passed / 18 failed / 2 ignored / 7401 filtered out`，exit 101，而不是原先假定的 5 tests。实际失败均来自该错误宽过滤带入的 Sound、旧 Render 期望、物理/VM、插件 manifest/workspace、Runtime06 status 与 diagnostics owner；不作为 Render01 5-test owner gate 失败。
- 真正的 5-test owner 集合由两个精确过滤构成：完整 module-path 过滤 `graphics::runtime::render_framework::submit_frame_extract::build_frame_submission_context::resolve_enabled_features::tests::` 命中 `resolve_enabled_features.rs` 的 3 tests，`render01_compiled_pipeline_runtime_metadata` 命中 `compiled_render_pipeline/tests.rs` 的 2 tests。不能用 `advanced_runtime_submission_flags`，因为它实际只命中其中 2 tests。原子计算后建立的 reservation `939c37bdbc00420d910b3ab6fd091cde` 在未 consume、未生成 job 时过期；等待期间 `zircon_runtime/src/graphics/tests/pipeline_compile/plugin_features.rs` 从 `95bc8052...` 漂移为 `ce6f0928...`，旧清单只剩 67/68 匹配。Render01 owner session `render01-f2-basic-scene-render-20260717` 已确认该变化为 async-compute fixture 修复，并正式授权完整 68-path 新聚合指纹 `786c6aed151cbea8063c76ded9ede7eb0b7c297467a828fe12179d46c24c5d34`；`4322914a...` 仅保留为宽波诊断历史。
- Windows managed 2-test current-source gate：reservation `aaf14cc15543432aa43a593f98cc7f4c`，job `9ca59803cf204dc0bc85c1849f25bfef`，run `a292084cde79437c884e3e1d8bd2fbf1`；reserve 与 consume/start 均逐文件复验完整 68-path `786c6aed...` 清单。`cargo test -p zircon_runtime --lib --locked --jobs 1 --color never render01_compiled_pipeline_runtime_metadata -- --test-threads=1` 执行 2/2 passed、0 failed、8239 filtered out，exit 0。
- Windows managed 3-test current-source gate：reservation `775d478614d24f2da4fd1d024a6d663f`，job `1ebe1346297c41c3b4e0d374bfde7c16`，run `80e37a5b155c4dea9381c18c9f37f2d5`；consume/start 再次逐文件复验同一 68-path `786c6aed...` 清单。`cargo test -p zircon_runtime --lib --locked --jobs 1 --color never graphics::runtime::render_framework::submit_frame_extract::build_frame_submission_context::resolve_enabled_features::tests:: -- --test-threads=1` 执行 3/3 passed、0 failed、8238 filtered out，exit 0。两个精确过滤合计 5/5 passed。
- Windows managed `descriptor` gate：reservation `419691b193d94656bf6203c9c523312b`，job `386e3d872b224189b488a0d37e564f34`，run `53f2e2c699974d9cb51bcbb9b3b040e5`；reserve 与 consume/start 复验 41/41 RuntimePlugin hard-cut owner/test files，source fingerprint `b61cef45ceca8aecd2d8fa6b3c9d1ed3de2041625a3dbc3f07963a56d3c161fb`。`cargo test -p zircon_runtime --lib --locked --jobs 1 --color never descriptor -- --test-threads=1` 完成 242 passed / 3 failed / 7996 filtered out，exit 101。三个 RED 均为 filter 带入的 lower owner：Render07 SSR history resource descriptor drift，以及 Runtime15 texture-descriptor/script-host plan anchor loss；已分别导入 Failure graph 为 `render/07/failure-2026-07-17-ssr-history-resource-descriptor-drift.md` 与 `runtime/15/failure-2026-07-17-descriptor-filter-plan-anchor-loss.md`，不归咎 RuntimePlugin descriptor hard cut，也不冒充该 gate GREEN。

## Current Blocker

- Coordinator01 一等 manifest 载荷阻塞已关闭；Render01 compiled-pipeline hard cut 已通过 `4322914a...` 源码切片的宽波编译，且保持字段私有。Render01 owner 授权的 `786c6aed...` 完整 68-path 清单已在两个 consume/start 边界复验，并完成精确 2+3 owner tests 5/5；不把全局 `plugin` 子串波冒充该 5-test gate。
- 18 个宽波失败是后续 package/full-wave 需要按 owner 收口的真实共享工作区负债，但不恢复 legacy pass order、兼容 alias/shim 或旧插件路径使其绿灯。`descriptor` gate 已证明 RuntimePlugin 41-path owner slice 在两次边界复验期间稳定，但当前由两个正式 lower-owner Failure lifecycle 阻断；先等待 owner fresh return，再重跑 `descriptor`，随后继续 `registration`、package compile 与插件工作区 gates。
- 任何 68 文件源码漂移都必须重新授权；当前只接受 `786c6aed151cbea8063c76ded9ede7eb0b7c297467a828fe12179d46c24c5d34`，不接受已拒绝或已过时的 `bec06a53...`、`3744b95e...`、`270fbc0b...` 或 `4322914a...`。

## Review

- 独立只读复审：Critical 0 / Important 0 / Minor 0。确认 21 个 typed fields/freeze/is_frozen 集合一致、无 legacy/compat/shim、descriptor/module registration 单源、exact integration 覆盖完整四阶段生命周期，并确认 3+2 精确过滤的 5-test 集合与 `786c6aed...` owner 授权口径一致。
- `descriptor` RED attribution 与两份 cross-plan handoff 经独立复审修正 `--exact` 完整测试路径后为 Critical 0 / Important 0 / Minor 0；本记录当前为 `in_progress`，不把 exact 1/1、精确 5/5、242/245 或静态门冒充 M3 全波次接受。
