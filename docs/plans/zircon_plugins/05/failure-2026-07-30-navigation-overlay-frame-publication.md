---
handoff_kind: failure
status: open
created_at: 2026-07-30
summary_slug: navigation-overlay-frame-publication
origin_plan: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
fixing_plan: docs/plans/zircon_plugins/05-navigation.md
origin_child_dir: docs/plans/zircon_editor/editor/05
fixing_child_dir: docs/plans/zircon_plugins/05
related_code:
  - zircon_plugins/navigation/editor/src/overlay.rs
  - zircon_plugins/navigation/editor/src/runtime_mirror.rs
  - zircon_plugins/navigation/editor/src/plugin.rs
  - zircon_plugins/navigation/runtime
tests:
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_plugin_navigation_editor -TargetDir E:/cargo-targets/zircon-navigation-overlay-source -SkipBuild
---

# Plugins05: Navigation overlay frame production is absent

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md`
- 来源执行切片：M4 shared viewport overlay-provider registry and Navigation provider integration gate
- 修复责任计划：`docs/plans/zircon_plugins/05-navigation.md`
- 交接原因：canonical NavMesh geometry、PIE agent mirror 与 provider factory 均由 Plugins05 拥有，Editor05 只拥有共享 viewport provider host。

## 失败现象与复现证据

Editor05 is implementing the shared viewport overlay-provider registry required by
`failure-2026-07-13-plugin-viewport-overlay-provider-runtime-wiring.md`. The existing
Navigation editor overlay can turn a supplied `NavigationGizmoSnapshot` and
`NavigationPieFrame` into `SceneGizmoOverlayExtract`, but repository-wide source inspection
finds no production publisher or consumer for `NavigationGizmoSnapshot` outside that overlay
module and its tests. The only registered PIE consumer is the agent-tick mirror, which has no
NavMesh area/link geometry.

## 最低共享层根因

The Navigation plugin does not publish one lifecycle-owned overlay frame containing both the
canonical NavMesh gizmo snapshot and the current PIE agent report. A viewport provider therefore
has no authoritative geometry input. Inventing an empty snapshot, a global cache, or a test sink
would make the new Editor05 registry appear wired while omitting the actual overlay.

## 架构修复验收

- Publish a typed, plugin-owned Navigation overlay frame from the canonical Navigation runtime
  source, with owner/session generation and capability lifecycle semantics.
- Reuse the existing `NavigationPieMirror` for agent data; do not create a second mirror or a
  process-global overlay cache.
- Make the registered Navigation provider consume that shared frame so every enabled viewport
  extraction contains NavMesh areas, agent paths, and avoidance vectors, and disabling the
  capability or ending PIE clears the extract on the next frame.
- Keep viewport ownership in Editor05: Plugins05 may supply a provider factory/data source but
  must not directly mutate editor viewport state.

### 必需证据

- A managed Navigation editor validation proves real NavMesh area, agent path, and avoidance
  extract presence from the registered provider, then their absence after toggle/capability or
  lifecycle shutdown.
- Record the source event/frame schema, ownership generation, exact command, raw terminal
  output, and review result in the Plugins05 child-plan return.

### Shared Host Dependency

The Editor05 host dependency is now implemented: active extension registries preserve executable
factories, atomically install `ViewportOverlayProviderRegistration`, and merge provider output into
the shared render/pointer interaction extract. Descriptor-only scene modes were removed, including
the former Navigation pseudo mode. Plugins05 must now contribute its real provider factory and
generation-owned canonical frame through that shared contract; it must not add a Navigation-specific
host call, global factory map, or retained-host exception.

## 禁止临时方案

- No empty or fabricated NavMesh snapshot, global singleton, duplicate PIE mirror, direct
  viewport mutation, test-only sink, compatibility alias, or fallback provider.

## 修复结果与回传

Open state: `待修复`; no provider/frame validation pass or fixed return is claimed.

Return a `fixed-*` record to Editor05 only after the provider receives the canonical frame and
the host-level packet test exercises the real data path. Editor05 retains the open host-wiring
failure until this dependency and the shared registry are both accepted.

## 产出记录与时间

| 时间 | 状态 | 完成项目与验证证据 |
| --- | --- | --- |
| 2026-07-30 22:20 +08:00 | `open / cross-plan dependency recorded` | 当前 Navigation registration 仅保存 provider ID；`core::plugin` active catalog 的 descriptor materialization 没有 provider factory 贡献，也没有把 active registry 安装到 host 的生命周期路径。该共享根因复用现有 Editor12 handoff，Plugins05 仍只负责 canonical frame 与真实 factory，不可引入直接 host 注入。 |
| 2026-08-01 | `host dependency implemented / Plugins05 open` | Editor05 已接通 executable extension/provider factory、capability lifecycle 与 shared interaction extract；Navigation descriptor-only pseudo mode 已删除。剩余缺口只在 Plugins05 canonical NavMesh+PIE frame 和真实 provider registration，仍不得用空 frame 或 PassThrough mode 代替。 |
