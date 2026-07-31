---
handoff_kind: failure
status: open
created_at: 2026-07-29
summary_slug: structure-guard-include-path-drift
origin_plan: docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md
fixing_plan: docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
origin_child_dir: docs/plans/zircon_runtime/text/09
fixing_child_dir: docs/plans/zircon_runtime/runtime/15
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/rhi_wgpu/ui_surface/tests.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/late_api_cleanup/f17_entity_path_lookup.rs
  - zircon_plugins/animation/runtime/src/lib.rs
  - zircon_runtime/src/animation/sequence/apply.rs
  - zircon_runtime/src/animation/sequence/target.rs
tests:
  - cargo +1.94.1 test -p zircon_runtime --lib tests::runtime_absorption::code_review_findings::late_api_cleanup::f17_entity_path_lookup::review_f17_entity_path_option_lookup_uses_get_verb --locked --jobs 1 --color never -- --exact --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --lib text::cache::tests::text_cache_indexes_keep_hot_lookup_and_eviction_work_constant --locked --jobs 1 --color never -- --exact --test-threads=1
---

# Runtime15：结构守卫 include 路径漂移

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md`
- 来源执行切片：Text09 cache index 受管精确编译门
- 修复责任计划：`docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md`
- 交接原因：两个失败点都是 Runtime15 拥有的 RHI UI 子模块测试和 late-API code-review 守卫没有随模块 owner hard cut 更新源码输入；Text09 只消费这些测试，不能在文本计划中建立例外路径。

## 失败现象与复现证据

受管命令

```text
cargo +1.94.1 test -p zircon_runtime --lib text_cache_indexes_keep_hot_lookup_and_eviction_work_constant --locked --jobs 1 --color never -- --test-threads=1
```

在 job `2f42664ec83b4d66a27a9f02671d5653`、run `02c936b32c8149e28eb633ed944e146c` 中以 `exit 101` 结束，目标测试未开始执行。已证实的守卫错误为：

- `rhi_wgpu/ui_surface/tests.rs:505` 从子目录使用 `include_str!("ui_surface.rs")`，实际 owner 在父目录 `rhi_wgpu/ui_surface.rs`；
- `late_api_cleanup/f17_entity_path_lookup.rs:8,10` 仍 include 已被 Plugins04 hard cut 删除的 `zircon_plugins/animation/runtime/src/sequence/{apply,target}.rs`。当前工作树将这两个已跟踪文件标记为 `D`，而 `zircon_plugins/animation/runtime/src/lib.rs` 已直接重导出 runtime animation API。现有 `Plugins04` open handoff `animation-sequence-caller-root-drift` 明确禁止恢复退役 `sequence` 根模块。

## 最低共享层根因

Runtime15 的结构和 review 守卫把源码文件路径当作稳定契约，却没有将路径契约收敛到当前父模块 owner 与 Plugins04 的 canonical crate-root/runtime animation ownership。守卫因自身的 `include_str!` 失败而阻断整个 `zircon_runtime` lib-test 编译，无法执行无关的 Text09 精确测试。

## 架构修复验收

- UI surface RenderDoc 守卫从 `rhi_wgpu/ui_surface/tests.rs` 读取实际父 owner `rhi_wgpu/ui_surface.rs`，不复制或移动 production source。
- F17 只检查仍存在的 canonical runtime animation apply/target owner，并在需要插件边界证据时检查 `zircon_plugins/animation/runtime/src/lib.rs` 的 crate-root re-export；不得再 include 已退役的插件 `sequence` 源文件。
- 不恢复 `mod sequence`、`sequence.rs` 或其 `apply.rs`/`target.rs`；保持 Plugins04 `animation-sequence-caller-root-drift` 的 hard-cut 禁令。
- Runtime15 的相关 structure/review 测试通过后，重新运行原始 Text09 受管命令；它必须至少越过这些 include 编译错误，才可进入 Text09 cache test 执行。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 不得恢复被 Plugins04 删除的 plugin animation sequence 模块，或以测试专用副本替代 canonical runtime source。
- 不得删除 F17 与 RenderDoc 守卫、降低断言，或把 Text09 的测试筛选改成绕过整个 lib-test 编译。

## 修复结果与回传

Current-source implementation (2026-07-29):

- `rhi_wgpu/ui_surface/tests.rs` now reads the actual parent owner with
  `include_str!("../ui_surface.rs")`; the RenderDoc source guard remains intact.
- F17 no longer reads the Plugins04 hard-cut `animation/runtime/src/sequence/*`
  files. It keeps `get_entity_by_path` assertions on canonical runtime apply and
  target sources, then verifies that the plugin crate root re-exports
  `apply_sequence_to_world` rather than reviving the retired module.
- Rust 1.94.1 scoped `rustfmt --check`, scoped `git diff --check`, and direct
  include-target resolution passed. The obsolete UI include and retired sequence
  include scans both return zero matches.

Managed validation ran from CPU reservation `8da59f6462274d8fa54babc7f2516d6a`
as job `a3c5aa2df56646578fc466dc9df461ea` / run
`185d3402f7ad4272933c1e4791c5dd00`, and exited `101` before the F17 test binary
started. The current lowest error is the active Editor11 canonical streaming owner:
`zircon_runtime_interface/src/serialization/text/canonical_writer.rs:298` needs
`CanonicalTupleVariant::finish(mut self)` before it can borrow `self.spool`
mutably. Existing Editor11 handoff
`docs/plans/zircon_editor/editor/11/failure-2026-07-29-canonical-text-streaming-output.md`
owns that repair. No F17 pass, Text09 upward pass, or failure return is claimed.

2026-07-29 follow-up: after the Editor11 canonical-writer repair appeared in the
shared worktree, managed CPU reservation `4da6742c201646099e4c339a6eda8f0f`
ran F17 as job `2c83bb33a2504c3dbb97c03b921c9fa8` / run
`315bd9d00b9e4960909f190ef7c394d6`. It compiled for roughly 35 minutes and
then exited `101` before the F17 test binary started. The next lower shared
compile owners are outside this Runtime15 structure-guard slice:

- `scene/dynamic_scene/session/{io,construction}` no longer exposes
  `construction::to_versioned_json_pretty_to` to its save callers;
- `rhi_wgpu/ui_surface/retained_cache.rs` calls the unavailable
  `retained_copy_byte_count` and imports private `render_backend::read_texture_rgba`;
- `plugin/native_plugin_loader/discovery_refresh` reaches private
  `discover::authority` items.

The F17 source repair remains present but unaccepted. No Text03/Text09 test,
WGPU product framebuffer capture, output record, or commit is claimed.

2026-07-29 verification correction: after Runtime15 repaired the DynamicScene
serialization facade, retained-cache helper/import boundary, and native-discovery
visibility boundary, managed reservation `35cf2346856244779492f91d40deaf5a`
ran job `191a43af42de46b18f2d3529a48a875a` / run
`2635a9abb3774c28ade162b7a25b5a98`. The lib test binary compiled and exited
`0`, but the former bare `--exact review_f17_entity_path_option_lookup_uses_get_verb`
filter selected zero tests (`9229 filtered`). This is compilation evidence only,
not an F17 pass. The frontmatter now records the source-true fully-qualified F17
and cache test names; their managed executions, the upward Text03/Text09 gates,
and the fresh WGPU product framebuffer remain required before return.

2026-07-30 support-first retry: the source-true F17 command was consumed as CPU
reservation `3975d1342ef74fae97b092ccd2d9f364`, job
`e89811bfe5a44d11815a3d0ec4832be7`, run
`71264c8644964a76a801e039bb348230`. It exited `101` before the test binary
started because `mesh/mod.rs` imported
`EnvironmentOnlyPbrBasePipelinePrewarmReport` while the existing type in
`mesh_pipeline_cache/ensure_pipeline.rs` was not re-exported by its parent
module. Runtime15 added the crate-visible parent-module re-export in
`mesh_pipeline_cache/mod.rs`; scoped Rust 2024 `rustfmt --check` and scoped
`git diff --check` pass. This repair awaits a new managed F17 run. No F17 pass,
Text09 pass, product framebuffer, output record, or commit is claimed.

2026-07-30 second support-first retry: the repaired source-true F17 command ran
under CPU reservation `d2fb0172c8b347ce91bbcedcc5798c35`, job
`a4eb0c99d0af45ee8b8394c0a7517855`, run
`5ca8e9f1d1654ad9bca00e148a81a3a1`. It compiled the current `zircon_runtime`
lib test for roughly twelve minutes, then exited `101` before the F17 test
binary started. The next two errors are owned by the active Plugins01 compiler
support session `plugins01-native-discovery-compile-boundary-r2-20260730`,
which carries forward the existing Plugins01/Frameworks04 native-discovery
failure chain:

- `discover/authority.rs` calls the private
  `NativePluginDiscoveryRefreshInput::root_scan()` from
  `discovery_refresh/contract.rs`;
- `discover_load_manifest.rs` passes the now-fallible TOML 0.9
  `Deserializer::new(...)` result directly to `DeserializeSeed::deserialize`.

Runtime15 does not cross-edit those leased native-plugin owners. Re-run this
same fully-qualified F17 command only after that lower shared compile boundary
returns. No F17 pass, Text03/Text09 pass, WGPU product framebuffer, output
record, or commit is claimed.

2026-07-30 accepted support-first gate: after the active Plugins01
native-discovery compiler repair returned, the source-true F17 command ran
under CPU reservation `d4b03d7c9fb143c98f9855678866832a`, job
`3d962990f2984ef2a288327ca0412bd0`, run
`3ef0c2c1b44645aaa5055db9d18555fa`. The managed run completed with exit `0`;
the fully-qualified filter executed one test and reported `1 passed; 0 failed`.
This accepts the Runtime15 F17 structure/review gate and confirms that the
mesh parent-module export and native-plugin compile boundaries no longer block
the lib-test binary. Text03/Text09 exact tests and the fresh WGPU product
framebuffer remain separate required gates; no product capture, output record,
or commit is claimed here.
