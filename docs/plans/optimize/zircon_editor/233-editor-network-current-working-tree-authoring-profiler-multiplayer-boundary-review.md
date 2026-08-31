---
title: Editor Network / Authoring / Profiler / Multiplayer Workbench 当前工作树复审
category: zircon_editor
report_id: Editor233
review_date: 2026-08-30
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/26-multiplayer-lobby-matchmaking-online-services-replication-network-emulation-pie-authoring-review.md
  - docs/plans/optimize/zircon_editor/100-editor-multiplayer-lobby-matchmaking-online-services-replication-network-emulation-pie-authoring-current-source-review.md
  - docs/plans/optimize/zircon_editor/148-editor-multiplayer-lobby-matchmaking-online-services-replication-network-emulation-pie-authoring-current-source-review.md
related_runtime_owner:
  - docs/plans/optimize/zircon_runtime/173-runtime-network-current-working-tree-authority-transport-session-rpc-replication-editor-boundary-review.md
related_code:
  - zircon_plugins/net/editor/src/plugin.rs
  - zircon_plugins/net/editor/src/authoring.rs
  - zircon_plugins/net/editor/src/tests/authoring_extensions.rs
  - zircon_plugins/net/editor
  - zircon_plugins/first_party_editor_catalog/src/catalog.rs
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/multiplayer
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback/online_sessions.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/online_sessions.rs
tests:
  - zircon_plugins/net/editor/src/tests/authoring_extensions.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/26-multiplayer-lobby-matchmaking-online-services-replication-network-emulation-pie-authoring-review.md
  - docs/plans/optimize/zircon_editor/100-editor-multiplayer-lobby-matchmaking-online-services-replication-network-emulation-pie-authoring-current-source-review.md
  - docs/plans/optimize/zircon_editor/148-editor-multiplayer-lobby-matchmaking-online-services-replication-network-emulation-pie-authoring-current-source-review.md
  - docs/plans/optimize/zircon_runtime/173-runtime-network-current-working-tree-authority-transport-session-rpc-replication-editor-boundary-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/NetworkPreviewSettings.cpp
  - dev/UnrealEngine/Engine/Source/Editor/NetworkPredictionInsights
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/NetDriver.h
  - dev/godot/editor/editor_network_profiler.h
  - dev/godot/editor/multiplayer
  - dev/godot/scene/main/multiplayer_peer.h
  - dev/godot/modules/multiplayer/scene_replication_interface.h
  - dev/bevy/crates/bevy_remote/src/http.rs
  - dev/Fyrox/fyrox-core/src/net.rs
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 1. 结论

当前 Net editor plugin 不是可用的 Network authoring/editor 产品，而是一个相当完整的描述符草图：`plugin.rs` 注册两个 surface，`authoring.rs` 注册两个 listener/route operation、四个 replication-schema operation、一个 asset type、一个 graph editor 和两个 graph nodes。对应测试只验证 registration report、payload schema、graph/palette descriptor 是否存在；没有执行 operation、读取 template、打开 document、运行 compiler 或连接 Runtime173 receipt。

当前工作树还存在五个硬缺口：所有声明的 `plugins://net/editor/*.zui`/默认 TOML 资源都不存在；operation 没有 factory/handler/document/transaction；Net 没有进入 `first_party_editor_catalog`；网络 diagnostics 没有 live provider；Multiplayer Workbench 的 Lobby/Matchmaking 页面是 collapsed 的固定 fixture，callback route 只返回静态文本。因此不能把 descriptor count、静态行或“simulation queued”当作 Unreal Network Preview/Network Insights、Godot Network Profiler 或真实 PIE client/server 的完成证明。

本报告新增 18 项 P1、8 项 P2、18 门资格门；没有复制旧报告的 P0。当前状态为 **14 Fail、4 Partial、0 Pass**。本轮仅 review，不修改 editor/runtime production code、ZUI 或 Cargo。

# 2. 当前工作树证据

## 2.1 选集与资源状态

| 范围 | files | lines | bytes | tests | ignored | fingerprint |
|---|---:|---:|---:|---:|---:|---|
| Net editor plugin（Rust/TOML） | **7** | **403** | **16,279** | **1** | **0** | current working-tree snapshot |
| Multiplayer Workbench lobby/matchmaking ZUI | **2** | **466** | **29,780** | **0** | **0** | current working-tree snapshot |
| Online-session feedback/navigation route | **2** | **273** | **11,465** | **0** | **0** | current working-tree snapshot |
| Runtime Net owner（关联证据） | **186** | **17,102** | **602,589** | **150** | **14** | `28336e23346b57dba0c9a41fd09d5ade1dcb56061933a43993a3f2d135d47c53` |

以下 URI 在当前工作树均未找到：

- `plugins://net/editor/authoring.zui`
- `plugins://net/editor/listener_config.zui`
- `plugins://net/editor/route_config.zui`
- `plugins://net/editor/replication_schema.zui`
- `plugins://net/editor/replication_schema.default.toml`

## 2.2 Registration 与 operation

- `zircon_plugins/net/editor/src/plugin.rs:25-39` 把 `plugins://net/editor/authoring.zui` 作为 template document URI，并调用 `register_net_authoring_workflows`；没有看到资源存在性检查、factory 注入或 host-ready validation。
- `zircon_plugins/net/editor/src/authoring.rs:43-73` 只构造 listener/route/schema command descriptors；`:76-124` 只声明三个 inspector URI、asset type 和 creation template；`:125-151` 只声明 graph editor、compile operation、palette nodes。descriptor 的 payload schema id 不等于 typed document schema 或 compiler artifact。
- `zircon_plugins/net/editor/src/tests/authoring_extensions.rs:11-126` 的测试只对 registration report 做 `assert`，没有读取五个 URI、执行 operation、检查 undo/redo、save/reopen、runtime install、diagnostics 或 error receipt。
- `zircon_plugins/first_party_editor_catalog/src/catalog.rs:41-54` 的 provider 只有 Navigation 和 Neural。Net plugin 即使可以独立返回 registration report，也不会由普通 EditorHost catalog 选中。

## 2.3 Workbench、routes 与运行时边界

- `workbench_extension_lobby_editor_workspace.zui:23-29` 根节点设置 `visibility = "collapsed"`；`:75-97,148-186,202-233` 的 Lobby_Default、Slot_Leader、Region_Auto、Player_01/02、8 slots/4 players/1 warning 和 dropdown options 全是固定 props。事件 route 没有 binding 到 Net document/session provider。
- Matchmaking workspace 具有同样 collapsed/fixed 结构：Playlist_Ranked、Queue_Solo、Rule_Latency、Bronze/Gold/Diamond/Backfill 和静态 queue/player/warning 数值不是 live matchmaking snapshot。
- `zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback/online_sessions.rs` 将 “Lobby simulation queued”“Matchmaking simulation queued” 等文本直接作为反馈；`.../navigation/specs/online_sessions.rs` 只列出静态 control/action routes，没有 operation factory、job id 或 Runtime event subscription。
- Editor 没有 Net session simulator、standalone server/client launcher、PIE multi-process attach、packet loss/reorder controls、network trace timeline 或 packet capture provider。Runtime173 的 root manager 也尚未产生可供 inspector/profiler 消费的 connection/session/RPC/replication receipts。

# 3. P1 差异与重构要求

| ID | 当前差异（证据） | 必须重构为 | 参考对照 |
|---|---|---|---|
| ED-NET-001 | `authoring.zui` 声明但缺失 | 提供版本化 root authoring template，启动时做 URI/resource admission | Unreal editor network preview settings |
| ED-NET-002 | listener inspector URI 缺失 | 真正的 listener document、typed fields、validation、error surface | Godot network profiler/editor panels |
| ED-NET-003 | route inspector URI 缺失 | HTTP/WS route schema、method/path/auth/timeout/budget editor | Unreal HTTP/online tooling |
| ED-NET-004 | replication schema inspector/default TOML 缺失 | stable schema asset、migration、default creation artifact 和 reopen proof | Unreal Iris schema tooling |
| ED-NET-005 | operation 只有 descriptor，无 factory/handler | 每个 operation 返回 typed job/receipt，支持 prepare/apply/cancel/failure | Zircon editor operation contract |
| ED-NET-006 | tests 只断言 descriptor | 添加 resource load、operation execution、negative/error、save/reopen/undo integration tests | Godot editor integration tests |
| ED-NET-007 | 没有 NetworkDocument/schema owner | 建立 document revision、stable object/field ids、schema registry、last-good generation | Unreal asset/editor model |
| ED-NET-008 | listener/route/schema mutation 无 transaction | 所有编辑进入 editor transaction、undo/redo、dirty/save/recovery journal | Godot editor undo/redo |
| ED-NET-009 | graph editor 只有 nodes/pins 描述 | graph body、stable node ids、type checking、compile job、artifact receipt、diagnostic spans | Unreal replication graph/Iris authoring |
| ED-NET-010 | 没有 scene/session/PIE command | 将 schema/listener/route attach 到 World/session/role，支持 standalone 与 PIE launch/attach | Unreal Network Preview |
| ED-NET-011 | diagnostics view 无 live provider | connection/session/RPC/replication/RUDP/download metrics 通过 bounded trace stream 投影 | Godot editor network profiler |
| ED-NET-012 | Workbench 根 collapsed、fixture 数据 | 根据 capability/session state 展示真实 lobby/matchmaking/connection rows，unsupported 必须显式显示 | Unreal session/PIE UI |
| ED-NET-013 | Workbench 没有 server/client emulator | 提供可控 local multi-process simulator、clock/latency/loss/reorder、teardown receipt | Godot MultiplayerPeer test harness |
| ED-NET-014 | route callback 只返静态 feedback | route dispatch 到 operation/job store，反馈包含 id、progress、diagnostic、terminal state | Zircon job/receipt model |
| ED-NET-015 | Net 没有 editor catalog/provider | 将 Net editor registration 加入 first-party catalog，并与 runtime activation generation 对齐 | Unreal module/editor registration |
| ED-NET-016 | 没有 permission/lease/audit | network credentials、endpoint、capture、session control 走 capability/permission/lease/audit | Godot editor security boundaries |
| ED-NET-017 | 没有 save/reopen/migration/recovery | canonical document、schema migration、last-good、journal recovery、source-control conflict resolution | Unreal asset transaction flow |
| ED-NET-018 | 没有 UI-to-runtime acceptance | 以同一 test project 验证 editor edit -> compile -> install -> PIE -> live profiler -> save/reopen | Unreal/Godot end-to-end editor tests |

# 4. P2 差异

| ID | 差异 | 重构方向 |
|---|---|---|
| ED-NET-019 | connection/session rows 不支持大规模数据 | virtualized table、incremental diff、stable row identity、filter/sort budget |
| ED-NET-020 | 没有 packet/RPC timeline | trace timeline、causal links、sampling、export/replay、privacy redaction |
| ED-NET-021 | schema diff 只可能依赖文本 | typed schema diff、compatibility matrix、migration preview、rollback |
| ED-NET-022 | QoS/security 没有 authoring surface | channel policy、TLS/pin/auth/key epoch、budget validation UI |
| ED-NET-023 | 没有 capture/replay | deterministic network capture, scrub, replay-to-World and mismatch diagnostics |
| ED-NET-024 | 多用户协作未定义 | document lease、conflict merge、presence、audit trail |
| ED-NET-025 | localization/redaction 未定义 | localized diagnostics and secret/token redaction in logs/profiler |
| ED-NET-026 | fault/scale UI 与自动化缺失 | test matrix for loss/reorder/restart/1K sessions and artifact-backed reports |

# 5. 资格门

| Gate | 状态 | 必须满足 |
|---|---|---|
| ED-NET-G01 | Fail | 五个声明的 URI 资源存在、版本可校验、加载失败可诊断 |
| ED-NET-G02 | Fail | Net editor provider 被普通 EditorHost catalog 选中并与 runtime capability 对齐 |
| ED-NET-G03 | Fail | 每个 operation 有 factory/typed payload/prepare/apply/cancel/terminal receipt |
| ED-NET-G04 | Fail | NetworkDocument、revision、stable ids、schema migration 和 undo/redo 可重放 |
| ED-NET-G05 | Fail | graph compile 产生 artifact/dependency/generation/diagnostic receipt |
| ED-NET-G06 | Fail | listener/route 实际绑定 Runtime173 root manager，关闭时有 quiesce receipt |
| ED-NET-G07 | Fail | replication schema 真正接入 World/Reflection/transport/auth session |
| ED-NET-G08 | Fail | standalone server/client 与 PIE multi-process 可由 Editor command 启停 |
| ED-NET-G09 | Fail | session simulator 支持 deterministic clock/loss/reorder/latency/restart |
| ED-NET-G10 | Partial | registration descriptors、graph palette 和部分 payload schema 已有，但没有 live provider |
| ED-NET-G11 | Fail | diagnostics/profiler 提供 bounded live trace、connection/RPC/replication metrics |
| ED-NET-G12 | Fail | Workbench 数据来自 snapshot/provider，fixture/collapsed 状态不再冒充成功 |
| ED-NET-G13 | Fail | save/reopen/migration/recovery 保留 document/artifact/runtime install 状态 |
| ED-NET-G14 | Partial | core route/feedback skeleton 存在，但 job id/progress/error/cancel 未闭合 |
| ED-NET-G15 | Fail | permission/lease/audit 和 secret redaction 对 endpoint/credentials/capture 生效 |
| ED-NET-G16 | Fail | packet capture/replay/fault/scale 结果可作为 artifact 打开和比较 |
| ED-NET-G17 | Fail | UI edit -> compile -> install -> PIE -> live network -> reopen 全链路自动化 |
| ED-NET-G18 | Partial | layout/route descriptor 能渲染部分壳，但资源缺失、root collapsed，不能发布 |

汇总：**14 Fail、4 Partial、0 Pass**。Partial 是描述符/路由骨架的局部证据，不代表 Editor Network 可用。

# 6. 建议重构顺序

1. **M0 Resource truth**：补齐五个 URI 资源和资源版本检查；将缺少 factory/provider 的入口显示为 Disabled/Unsupported，并移除静态 queued success 文案。
2. **M1 Document/operation**：建立 NetworkDocument、schema registry、stable ids、typed operation factory、transaction/undo/redo、save/reopen/migration/recovery。
3. **M2 Runtime attach**：让 first-party editor catalog 注册 Net；operation 生成 Runtime173 activation/session/World attach receipt，提供 standalone/PIE client-server launcher 和 cancel/teardown。
4. **M3 Profiler/simulator**：加入 live bounded trace、connection/session/RPC/replication/RUDP/download panels，提供 deterministic clock、loss/reorder、capture/replay 和 fault receipt。
5. **M4 Workbench product**：将 Lobby/Matchmaking rows、rules、regions、players、warnings 和 actions 改成 provider-backed virtualized projections；collapsed 只由 capability/session state 控制。
6. **M5 Qualification**：执行 UI automation、save/reopen、PIE/standalone、fault/scale/soak，并把每次结果绑定到 document/artifact/runtime generation。

# 7. 参考引擎对照结论

- Unreal 的 Network Preview、NetDriver/NetConnection、ReplicationGraph/Iris tooling 和 NetworkPrediction Insights 把 editor command、PIE process、runtime trace、replication schema 连接到可观察的 session/artifact；Zircon 当前只有 registration descriptors 和静态 Workbench。
- Godot 的 editor network profiler、MultiplayerPeer/SceneMultiplayer/replication interfaces 展示了 peer/session/authority 在 editor 与 runtime 间共享的边界；Zircon 的 Net provider 未进入 editor catalog，session simulator 也不存在。
- Fyrox 的 Rust network module、Bevy Remote HTTP/remote commands 可参考轻量 document/command 与 bounded response，但不能替代 NetworkDocument、World attach、PIE 和 profiler 资格。

# 8. 验证边界

本轮只读取当前源码、TOML、ZUI 和 catalog 路径；未运行 Cargo、Editor、UI automation、PreviewWorld、PIE、standalone server/client、真实 socket/TLS、save/reopen、fault、scale、soak 或 benchmark。Editor233 必须与 Runtime173 同步复核；编辑器描述符测试通过也不能把缺失资源、静态 fixture 或 no-op operation 标记为 Closed。
