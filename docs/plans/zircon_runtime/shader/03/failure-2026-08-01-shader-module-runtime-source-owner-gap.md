---
handoff_kind: failure
status: open
created_at: 2026-08-01
summary_slug: shader-module-runtime-source-owner-gap
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/shader/03-module-imports-and-cross-references.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/shader/03
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/asset/project/manager/scan_and_import/shader_import_dependencies.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_shader_source.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs
  - zircon_runtime/src/graphics/shader/template/module_registry.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_shader_permutation_manifest.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/permutation_registry.rs
tests:
  - render_product_source_only_cross_module_material
  - render_product_plugin_cross_module_material
---

# Shader03: product runtime has no source owner for source-only or plugin modules

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：2026-08-01 current-source plan/code review convergence
- 修复责任计划：`docs/plans/zircon_runtime/shader/03-module-imports-and-cross-references.md`
- 交接原因：Shader03 owns module source registration, dependency resolution, template assembly and the product acceptance boundary; Performance01 must not create a second shader-module source bridge.

## 失败现象与复现证据

- `ProjectManager::scan_and_import` can append a source-only import asset to the referencing shader's `dependency_ids`, so hot-reload dependency propagation is present.
- ResourceStreamer module collection only follows imports with `redirect`; `resource_streamer_accessors.rs` skips `redirect == None`, so the source-only module is not supplied to `ShaderModuleRegistry` for product assembly.
- Plugin manifests and `zircon_shader_prewarm` merge `shader_modules` into a hash overlay, but the product runtime registry has no plugin catalog source bridge.
- Existing tests prove project dependency indexing, registry algorithms, redirect product supply and prewarm hashing separately. None assembles a project surface through a source-only module or plugin module in the real ResourceStreamer/template path.

## 最低共享层根因

Shader03 has two metadata authorities but no product runtime source authority. A dependency id or content hash identifies invalidation; it does not provide the WGSL module body required by template assembly. Until prepared shader/runtime state carries resolved `import_path -> module source` bindings from project and plugin catalogs into the unique `ShaderModuleRegistry`, the advertised cross-module product contract is incomplete.

## 架构修复验收

- Define one runtime-owned resolved module-source binding containing import path, source asset/package identity, source text, content hash and diagnostic origin.
- Carry project source-only imports from resolved project artifacts into prepared shader state; do not infer source by rescanning paths during rendering.
- Bridge plugin package shader modules into the same runtime source owner with explicit precedence and redirect diagnostics; prewarm remains a consumer, not the source authority.
- Keep built-in, project, redirect and plugin modules in the existing `ShaderModuleRegistry`; do not create parallel registries.
- Add product tests that assemble and Naga-validate or WGPU-render a project surface through a source-only module and through a plugin module. Assert module body inclusion, dependency hash participation and actionable missing-source diagnostics.

## 禁止临时方案

- 不得把 `dependency_ids`、prewarm hash 或 manifest presence 当作模块源码已经进入产品路径的证据。
- 不得在 ResourceStreamer 中按磁盘路径临时重扫 WGSL，或建立 project-only/plugin-only 第二注册表。
- 不得只扩充 registry 单测、redirect 测试或 prewarm 测试后关闭本记录。
- 不得把 source-only import 静默改成必须填写 redirect；两种声明形态的产品语义必须按计划明确实现。

## 修复结果与回传

2026-08-01 静态审阅已确认依赖索引、redirect 产品供给和 prewarm overlay 各自存在，同时确认 source-only 与插件模块的产品源码供给缺失。相关 Shader 实现路径当前存在并行脏改，本轮未覆盖源码，也没有 current-source Cargo/WGPU 通过结果。

2026-08-01 前向修复实现完成：ResourceStreamer 将同步阶段发布的 shader `dependency_ids` 递归预备为源码，并在模板供给时按相同 id 链收集 include。`ShaderModuleSourceBinding` 是项目 asset 和插件 package 共用的唯一 runtime binding，保留 owner identity、import path、source text、blake3 hash 和 diagnostic origin；项目 `PreparedShader` 与 binding 以 `Arc<str>` 共享正文，再转换为仍保留这些元数据的 `ShaderTemplateInclude`。native package 不会在冻结 load projection 时读取整个 discovered catalog，只在生成实际 runtime registration report 时以受限 package-relative 路径解析该包模块，并施加单模块 4 MiB、每包 64 模块及每包总计 16 MiB 预算；linked plugin 可以提供同型 binding。RuntimeExtensionRegistry、GraphicsModule 与 SceneRenderer 在初始化期把 binding 传给 ResourceStreamer，渲染时无包路径重扫；现有 ShaderModuleRegistry 仍是唯一解析器，并按 `redirect > project source-only > plugin` 选择同名模块。新增 source-only 项目模块和 plugin 模块的真实 ResourceStreamer -> template -> Naga 用例，plugin 用例同时 WGPU shader-module validation，source-only 用例验证对同名 plugin binding 的覆盖与 content hash；同时覆盖 linked plugin 注册、native 缺失模块诊断延迟、64 模块与 16 MiB 总量预算，以及跨 package 同 token 的冲突诊断。`cargo test --locked` 尚未进入编译：共享 `Cargo.toml`/`Cargo.lock` 当前不一致，Cargo 在解析阶段要求更新锁文件；Runtime04 owns that baseline, so this failure does not rewrite it。

Open state: `implementation is complete; cargo metadata --locked --no-deps now resolves the shared workspace, but that read-only check neither compiles nor executes the source-only/plugin product tests. Keep this failure open until a managed Windows locked product result passes, then return for accepted closeout.`

2026-08-02 forward repair: review found that a FeatureExtension package, and an
ordinary package whose only runtime entry is an optional feature, had no
ordinary runtime registration report to own their package shader bindings.
Active runtime feature reports now carry those bounded package-relative
bindings only when there is no primary runtime report. Packages without a
runtime feature return before source I/O; packages with a primary runtime
module keep the single ordinary-report owner. Runtime assembly deduplicates
only identical `(owner_id, import_path, content_hash)` copies from multiple
features, preserving same-token conflicts for the existing ResourceStreamer
diagnostic. Focused coverage proves both forms reach graphics registration and
that active-feature duplicates collapse once. Post-fix independent review:
`Critical 0 / Important 0 / Minor 0`. This is a forward repair on the
integrated snapshot; the handoff remains `open` pending managed Windows locked
product evidence.

### 2026-09-19 successor static source receipt

Successor Session `failure-roll-01a084c8-shader03-module-source-owner-r1`
sealed the current project/plugin module-source owner paths after correcting
the stale related-code path to
`zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs`.
The first checker ticket `6772c9d1087d4a3f8df9a094b4cf7484` failed only because
its checker required a non-existent prose anchor; that evidence is retained.
Corrected ticket `e01a65ca09934ca1b8a65224001175a1` (request
`failure-roll-01a084c8-shader03-module-source-owner-20260919-r2`) used the
actual `shader_modules.get(&import_path)` and
`DuplicateShaderModuleContentHash` merge anchors. Coordinator job
`34482a71be50417a998128e765b52050` / run
`e01a65ca09934ca1b8a65224001175a1` exited 0 with
`SHADER03_MODULE_SOURCE_OWNER_CURRENT_SOURCE_CONTRACT_PASS`; cleanup completed.
This is static current-source evidence only. Managed source-only/plugin Cargo,
Naga/WGPU product assembly, independent review, fixed return, and closeout
remain pending.

### 2026-09-21 independent current-source review receipt

Reviewer Session `review-shader03-module-source-owner-r1` rechecked the
current immutable Shader03 source-owner slice without editing production code
or absorbing the neighboring foreign worktree changes. The seven production
paths from corrected ticket `e01a65ca09934ca1b8a65224001175a1` still match its
manifest: `shader_import_dependencies.rs`=`afa91b0b8537e438be6f5d3dd36ce9131c35fc43670e54ff20827917b371b649`,
`resource_streamer_ensure_shader_source.rs`=`a2045ce61ff0675255e80278b3cd258a974ef865294aedf16a38c55a06cb138b`,
`resource_streamer_accessors.rs`=`9189be537c0f5781043b6b17dc393fd88d4cd63d8f54f703275020c6b27c58d3`,
`module_registry.rs`=`010d76a6d3cc6d60a3f35b7ee7d26da993b97d47ef9bc5fd1ef2f96af8e6bb11`,
`plugin_shader_permutation_manifest.rs`=`a0dcc7ff844b0f1afb1939b4bf3934589e2ab3007190286a483c6f68474028bb`,
`permutation_registry.rs`=`5c70a8253dba23ce1bac042e8010579f07a164c892befb5ea10840760d0f9130`.
The failure document and plan were also frozen for this receipt at
`26dbe46b7283eb3b38b10fd7ead388666ca010ad95f11895f961b28249aa59e4` and
`3848b2a96546bed815fa053941c65696ebaf79fe7f8c7ca0cae0c60697f23e01`,
respectively, before this append.

Static review evidence:

- `rustfmt +1.94.1 --edition 2021 --config skip_children=true --check` passed
  for all six Rust production files in the corrected ticket; scoped
  `git diff --check` passed (only the repository's existing LF/CRLF warnings).
- Source probe passed as `SHADER03_MODULE_SOURCE_OWNER_REVIEW_PASS`. It verifies
  the single `ShaderModuleSourceBinding` carries owner/import/source/hash/origin;
  source-only dependency IDs and redirect imports are both prepared recursively;
  ResourceStreamer emits plugin bindings first and project bindings afterward so
  `redirect > project source-only > plugin` precedence is explicit; no render-time
  filesystem read exists in the accessor; the single registry retains topology,
  cycle, unknown-module, dependency-hash, and diagnostic-origin behavior; the
  prewarm merge remains a hash consumer and preserves duplicate-hash diagnostics;
  and the construction map rejects hash mismatches and cross-owner token
  collisions.
- Read-only supporting paths confirm the native package source loader enforces
  64 modules, 4 MiB per module, 16 MiB total, package-relative `.zshader`/`.wgsl`
  paths, canonical-root containment, and delayed source I/O. The current product
  test child contains source-only and plugin ResourceStreamer→template→Naga tests,
  a WGPU shader-module validation call, cycle diagnostics, redirect diagnostics,
  and project-over-plugin same-token assertions. These supporting paths are not
  attributed to the Shader03 primary Session because they are outside its sealed
  manifest and may have foreign owners.

Independent review result: Critical=`0`, Important=`0`, Moderate=`0`. This is a
current-source/static review only. The required managed Windows `--locked`
source-only/plugin Cargo result, Naga/WGPU product execution, and any upward
acceptance remain pending; the prior external `E:\Git\zr_vm` dirty-worktree
admission blocker is not a test result. Canonical `fixed-*`/return, closeout,
and WeCom notification therefore remain pending and this failure stays `open`.

## 2026-09-26 successor intake (failure-roll-01a084c8-shader03-module-source-owner-r2)

- The archived r1 source-owner lifecycle and its passed static ticket remain historical evidence only. Successor r2 owns this failure document under a fresh lease against base SHA-256 `37a0bdffaa7d0b999546d85cc56e72a6d9e9b8d651eb856f85204fa805c59f04`; no production source path is leased or edited.
- The six current production paths were rehashed before intake. Current SHA-256 values are: `shader_import_dependencies.rs` `3c037ffadb7f7a94f8ac2b578fd66ec5444831c54fde388974692a5926823b87`; `resource_streamer_ensure_shader_source.rs` `a2045ce61ff0675255e80278b3cd258a974ef865294aedf16a38c55a06cb138b`; `resource_streamer_accessors.rs` `9189be537c0f5781043b6b17dc393fd88d4cd63d8f54f703275020c6b27c58d3`; `module_registry.rs` `010d76a6d3cc6d60a3f35b7ee7d26da993b97d47ef9bc5fd1ef2f96af8e6bb11`; `plugin_shader_permutation_manifest.rs` `a0dcc7ff844b0f1afb1939b4bf3934589e2ab3007190286a483c6f68474028bb`; `permutation_registry.rs` `5c70a8253dba23ce1bac042e8010579f07a164c892befb5ea10840760d0f9130`. The first path differs from the historical ticket because a foreign optimization batch appended test modules and capacity helpers; its existing attribution and dirty bytes are preserved, not absorbed.
- This is a source-bound static successor only. The corrected source-contract ticket and r1 review are not reused as current-source or dynamic acceptance. Managed Windows locked source-only/plugin Cargo, Naga/WGPU product assembly, upward acceptance, canonical fixed return, coordinator closeout, and WeCom notification remain pending. The failure stays open.

### r2 successor independent review receipt

Reviewer `/root/review_editor03_gizmo_private` re-read snapshot 3939 (SHA-256 `1b231b05b61cea9f73f6599ba25e580bad87567b8433897d1e6de6e95b4c82d0`). Archived r1 evidence is explicitly historical and non-reused; the r2 lease base `37a0bdffaa7d0b999546d85cc56e72a6d9e9b8d651eb856f85204fa805c59f04` is clear. All six current production hashes are listed; `shader_import_dependencies.rs` retains its foreign optimization/test drift without absorption and the remaining five match their historical values. Independent review result: **Critical=0 / Important=0 / Moderate=0**. The static source probe remained green, while managed locked Cargo/Naga/WGPU, upward acceptance, fixed return, closeout, and WeCom remain pending. The failure stays open.
