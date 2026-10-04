---
handoff_kind: failure
status: open
created_at: 2026-08-01
summary_slug: physics-debug-overlay-provider-missing
origin_plan: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
fixing_plan: docs/plans/zircon_plugins/03-physics.md
origin_child_dir: docs/plans/zircon_editor/editor/05
fixing_child_dir: docs/plans/zircon_plugins/03
plan_link_mode: child_record_only
related_code:
  - zircon_plugins/physics/editor/src/plugin.rs
  - zircon_plugins/physics/editor/src/tests.rs
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_overlay_providers.rs
tests:
  - physics editor provider registration and capability lifecycle tests
  - host toggle to shared viewport interaction extract product test
---

# Plugins03: Physics debug overlay lacks an executable provider

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md`
- 来源执行切片：Editor05 executable scene-mode hard cut current-source review
- 修复责任计划：`docs/plans/zircon_plugins/03-physics.md`
- 交接原因：Editor05 只能删除不可执行的伪 scene mode；Physics 调试几何、capability 生命周期和 overlay provider 必须由 Plugins03 提供。

## 失败现象与复现证据

The Physics editor registered `physics.debug_overlay.mode` as descriptor-only viewport metadata.
It supplied neither a `SceneModeRegistration` factory nor a
`ViewportOverlayProviderRegistration`, so the entry could never execute or publish collision
geometry. Editor05 removed this pseudo mode during the executable scene-mode hard cut; keeping a
PassThrough factory would only disguise the missing Physics behavior.

## 最低共享层根因

The old contribution contract treated toolbar metadata as proof of an executable mode. Plugins03
did not own a provider that converts canonical Physics debug geometry into the shared viewport
extract, so the UI descriptor had no production behavior behind it.

## 架构修复验收

- Register a Physics-owned `ViewportOverlayProviderRegistration` backed by the canonical debug
  geometry generation and capability lifecycle.
- Route the existing toggle operation to `ViewportCommand::ToggleOverlayProvider`; do not model a
  debug overlay as a base scene mode.
- Prove enabled/disabled extract publication through the host's shared render/pointer interaction
  extract, including stale-frame clearing.

## 禁止临时方案

No descriptor-only mode, empty PassThrough factory, fabricated geometry, global cache, direct
viewport mutation, compatibility alias, or test-only provider.

## 修复结果与回传

Open state: Editor05 已移除 `physics.debug_overlay.mode` 伪注册，Plugins03 尚未交付真实
`ViewportOverlayProviderRegistration`、共享 extract 产品证据或 focused Cargo GREEN。本记录保持
`open`，不得把“不可点击的伪入口已删除”误记为 Physics overlay 已完成。

| 日期 | 项目 | 状态 | 证据 |
| --- | --- | --- | --- |
| 2026-08-01 | Physics pseudo scene-mode hard cut | open | Editor05 executable registry migration removed `physics.debug_overlay.mode`; command/menu/view remain, but no production Physics provider exists. Plugins03 owns the canonical geometry/provider repair and product gate. |
| 2026-09-26 | Plugins03 current-source successor r2 | waiting_validation | Successor Session `failure-roll-01a084c8-plugins03-physics-overlay-r2` transferred the archived 13-path scope; source-only snapshot `3886` was followed by authoritative post-receipt snapshot `3887` (failure doc plus current Physics runtime/editor provider, mirror, tests, and shared render overlay contract). The exact source probe printed `PLUGINS03_PHYSICS_OVERLAY_CURRENT_SOURCE_PASS`; `cargo +1.94.1 fmt --manifest-path zircon_plugins/physics/runtime/Cargo.toml -- --check` and the editor equivalent both passed. A managed Cargo ticket submission with request `plugins03-physics-overlay-current-cargo-20260926-r2a` was rejected during admission as `validation_ticket_external_deadline_exceeded` while discovering external Git input, so no Cargo job/run or dynamic GREEN evidence exists; the failure remains open. |

### Successor snapshot manifest and binding

The archived owner scope contains thirteen paths: this failure record plus the twelve source
paths below. The coordinator post-receipt snapshot `3887` binds the failure-record hash
`dd1343721f10e52f05c8d75d4182b0bd4799489515c8402e7485b28a5e8635d9` and the following current
source hashes; `3886` is the pre-receipt source-only boundary and is not the authoritative document
boundary. The original four `related_code` entries remain the upstream handoff links; this manifest
is the complete successor ownership and provenance set.

| owned path | SHA-256 at snapshot 3887 |
| --- | --- |
| `zircon_plugins/physics/editor/src/lib.rs` | `e18a3e26ced2e8ed1013b17673e02f60bac105298ff0136070c5b07a56ef598e` |
| `zircon_plugins/physics/editor/src/overlay.rs` | `5a5f6f4291ad9d0631519d66945ea304532e9dca42937e174abd10b966ca9fc5` |
| `zircon_plugins/physics/editor/src/overlay_provider_tests.rs` | `4dfa331d583dcc69d6908506c21a8e7d7ccd58a972686f18c08c795a9a19b66a` |
| `zircon_plugins/physics/editor/src/plugin.rs` | `6eed38bcfc3027868a07a7ec744ab94e275de394bd5cdbe882094545342958c0` |
| `zircon_plugins/physics/editor/src/runtime_mirror.rs` | `89b03d3304715e2ea022fe65c4d698d1614341684a25b59e955ac8394e8e343b` |
| `zircon_plugins/physics/editor/src/viewport_overlay_provider.rs` | `77d548d8382ae5cef95e8bcbc13882516e9349cb74117c7ac6fdcc4939c60ee1` |
| `zircon_plugins/physics/runtime/src/lib.rs` | `4e50a52002b2ecb5d29254f0f3eeaa2e228dc54d3a4ac8f2b31e922c7bfe09d9` |
| `zircon_plugins/physics/runtime/src/overlay_frame.rs` | `29124ab1e1a191b25a351671150533e6a7a7ef300744449c55991f72a40875f1` |
| `zircon_plugins/physics/runtime/src/overlay_frame_tests.rs` | `952953e9ff4ba8da823c9aed7f03d3c1a9a63f1e52e41d4f0bfb0c069f3da265` |
| `zircon_plugins/physics/runtime/src/plugin.rs` | `6332a668637fbc14f4190611b812a937c1cf5be11f93394fb8ced01732136ea5` |
| `zircon_plugins/physics/runtime/src/runtime_system.rs` | `7ea40e5d5f59012b15fec187e86c4005d983c0ae8f67db3048f2e1edf59ae79d` |
| `zircon_runtime/src/core/framework/render/overlay.rs` | `c36a93e68a92becabcb2780d94cf5fa2f22121e07243886f17fc62f1a6acb8a0` |

Snapshot `3888` is the post-manifest correction seal for this handoff record; it changes only
the failure-record prose and leaves all twelve source hashes above unchanged. The next
coordinator snapshot records the resulting document hash without changing the source boundary.

### Independent review

The reused coordinator-efficiency reviewer completed an independent recheck against snapshot
`3890` and the current working tree: Critical `0`, Important `0`, Moderate `0`. The 13-path
scope (failure record plus 12 source paths), all manifest hashes, the `3886` → `3887` → `3888`
→ `3889` → `3890` wording/provenance chain, static marker, and formatter receipts are coherent.
The Cargo admission rejection remains the only validation result; no dynamic GREEN, return, or
closeout is inferred. Snapshot `3891` is the post-review document seal.
| 2026-09-11 | Plugins03 provider source slice | waiting_validation | Source snapshot `3395` freezes the Physics runtime mirrored-event capture, PIE stale-frame mirror, provider registration, shared `SceneGizmoOverlayExtract` geometry/pick shapes, and `ToggleOverlayProvider` routing. The exact managed Windows batch `cargo +1.94.1 test -p zircon_plugin_physics_runtime -p zircon_plugin_physics_editor --lib --locked -- --test-threads=1 --nocapture` was rejected before Cargo started with `validation_ticket_external_worktree_dirty`: external `E:\\Git\\zr_vm` must be committed clean first. No dynamic GREEN evidence exists, so this failure remains open. |
