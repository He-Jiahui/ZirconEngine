---
handoff_kind: failure
status: open
created_at: 2026-07-19
summary_slug: kira-send-frame-capture-routing
origin_plan: docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
fixing_plan: docs/plans/zircon_plugins/02-sound.md
origin_child_dir: docs/plans/zircon_runtime/render/01
fixing_child_dir: docs/plans/zircon_plugins/02
plan_link_mode: child_record_only
related_code:
  - zircon_plugins/sound/runtime/src/kira_bridge/graph_compile.rs
  - zircon_plugins/sound/runtime/src/kira_bridge/graph_compile/routes.rs
  - zircon_plugins/sound/runtime/src/kira_bridge/manager.rs
  - zircon_plugins/sound/runtime/src/kira_bridge/manager/graph.rs
  - zircon_plugins/sound/runtime/src/kira_bridge/manager/graph/transaction.rs
  - zircon_plugins/sound/runtime/src/tests/kira_bridge/graph/routing.rs
tests:
  - post_effect_send_obeys_target_bus_gain_mute_and_parent_gain
  - master_track_gain_is_applied_once_to_direct_and_send_paths
  - active_graph_sync_updates_the_rendered_send_for_parent_gain_changes
---

# Plugins02: Kira send frame-capture routing current-source RED

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md`
- 来源执行切片：Render01 frame-capture route gate
- 修复责任计划：`docs/plans/zircon_plugins/02-sound.md`
- 交接原因：Sound Kira graph compile/route installation boundary 是三项 send routing 失败的最低 owner。

## 失败现象与复现证据

- Managed job `99687c8d6c584399aa727b09e121cdc1`, run `dc0d773d6c1e4b8b9b2b610114f7867c`, completed/released with exit `101`.
- Exact frame-capture gate executed `4` tests: `1 passed / 3 failed / 325 filtered`.
- The three failures show that post-effect send contribution was absent, master gain did not scale direct/send paths exactly once, and active graph resync did not update the rendered send after parent-gain changes.
- Build completed in `7.62s`; tests completed in `0.06s`. This is accepted as the Sound RED baseline and is not a Render-owned failure.

## 最低共享层根因

The failure belongs to the Sound Kira graph compile/route installation boundary. Render01/F2 and Shader06 must not patch or absorb these paths.

## 架构修复验收

- Fresh canonical Rust 1.94.1 focused route GREEN on the exact current source.
- Fresh plugin broad/product GREEN and current lock evidence.
- Independent review Critical/Important `0/0`.
- Coordinator-managed atomic milestone commit with immutable SHA and shared staged count `0`.

## 禁止临时方案

Render01/Shader06 不得吸收 Sound 路由修复，不得弱化三项 frame-capture 失败断言，也不得用非 current-source 产物替代 focused gate。

## 修复结果与回传

This record remains `open` until all current-source acceptance evidence is recorded.

### 2026-09-01 managed validation retry

The current source and the three route regressions are present in the immutable HEAD
snapshot. Rustfmt and scoped diff checks passed, but the managed validation copy
`980117cb813547d3b3de05f17fc5c653` was rejected before closure planning or Cargo
startup. The durable copy status was `failed`/`removed` with:

- `errorCode`: `unmanaged_artifacts_detected`
- `errorStage`: `artifact_governance`
- `errorPath`: `E:\\cargo-targets\\zircon-engine\\scratch\\pinned-debug-ag09aq02`

The path belongs to another producer and was not modified or removed by this Session.
No focused test result is claimed from this attempt. The Kira implementation remains
source-ready, while the exact focused and broad managed Cargo gates stay pending an
artifact-governance clean window.

### 2026-09-11 rolling repair continuation

The rolling repair resumed under the stable coordinator Session
`failure-roll-01a090ae-plugins02-kira-routing-r1` with the original seven-path write
scope.  The current source was rechecked against the Session attribution before
validation: `graph_compile.rs` `40e9de7f9bbd724c1e1045559815510ead4f356c22a9bcad6f1997744c0ee983`,
`graph_compile/routes.rs` `2bc52f097a2b403237090205625a696ede4efc92dccbc25c7ef6fb9aafb8c33c`,
`manager.rs` `2bb205b80c11589d61c9cbc572c733504cf1d7cf50885ebcade84c3acaccaafd`,
`manager/graph.rs` `f6aa449580a723178e5cc8ffe22f1b33c9854b43e984006bc6b404683e5f1aca`,
`manager/graph/transaction.rs` `72d900943fe1da5ed4873c20d7fe756d0be4eabcc3f76ab3de3b4e8b795f1f36`,
and `tests/kira_bridge/graph/routing.rs`
`a2a4f6df9d8efcff1d0dabfc52e8b589175beb47f5ee4c1b167aa627c061f19e`.

The three original RED test names are each present exactly once in the current
`routing.rs`.  A local source-contract check passed as
`PLUGINS02_KIRA_ROUTING_STATIC_PASS`; it confirmed the expanded post-effect route
calculation, effective parent/mute gain propagation, send-target and track-send
sync actions, Kira send-track installation, global/master gain composition, and
staged send/track installation.  This is static evidence only and does not replace
the required current-source focused and broad managed Cargo runs.

The prior managed retry remains non-reusable: ticket/copy `980117cb813547d3b3de05f17fc5c653`
was rejected at artifact governance with `unmanaged_artifacts_detected` for the
foreign path `E:\\cargo-targets\\zircon-engine\\scratch\\pinned-debug-ag09aq02`.
The current Session therefore submits fresh managed static, focused, and broad
validation tickets; no dynamic GREEN result is claimed until their source and
configuration bindings pass.

The fresh managed static ticket `ddc81648724b4006a342068fff6a27e4` was accepted
with the seven-path source manifest (`6019fd81698fefe237bce1eeab03f7f6771a32beea2e1e7fa4bc145e62520e1e`)
and remains queued; its Python contract command has not yet produced a terminal
result.  The focused Cargo submission (`tests::kira_bridge::graph::routing`) and
the plugin-broad Cargo submission were both rejected at admission with
`validation_ticket_external_worktree_dirty` for the unchanged foreign worktree
`E:\\Git\\zr_vm`; neither started Cargo and neither is a dynamic pass or failure
of this source.  The required gates remain pending a clean external worktree.

### 2026-09-18 static ticket terminal result

The static ticket `ddc81648724b4006a342068fff6a27e4` subsequently reached terminal
`passed`. Its managed job/run was `f902a6a7ad0a440da2625365f310e470`, exited `0`,
and emitted `PLUGINS02_KIRA_ROUTING_STATIC_PASS` with no coordinator blockers.
This terminal receipt supersedes the earlier `remains queued` wording above; it
is still static contract evidence only. The focused and plugin-broad Cargo gates
remain pending because their admission was rejected for the unchanged foreign
worktree `E:\\Git\\zr_vm`.

### 2026-09-19 current-source reconciliation receipt

- The successor Session `failure-roll-01a084c8-plugins02-kira-routing-r2` rechecked all six source/test files against the predecessor static ticket manifest. Their current SHA-256 values are unchanged from manifest `6019fd81698fefe237bce1eeab03f7f6771a32beea2e1e7fa4bc145e62520e1e`.
- Therefore the passed static ticket `ddc81648724b4006a342068fff6a27e4` (job/run `f902a6a7ad0a440da2625365f310e470`, exit `0`, marker `PLUGINS02_KIRA_ROUTING_STATIC_PASS`) is reusable as current-source static evidence only.
- Read-only inspection of `E:\Git\zr_vm` still shows foreign tracked and untracked modifications. The focused route Cargo gate and plugin-broad Cargo gate remain admission-blocked with `validation_ticket_external_worktree_dirty`; neither produced a dynamic result.
- The failure remains open. Current-source focused/broad Cargo validation, upward Render01 acceptance, independent Critical/Important/Moderate review, canonical `fixed-*`/return records, and coordinator closeout are pending a clean external dependency revision.

### 2026-09-21 independent review and current acceptance boundary

- Independent review Session `review-plugins02-kira-routing-r2` audited the current six-file production/test call chain at HEAD `6b4bc86089cb4464f850136c8079e90cb6513ebc` and baseline epoch `611`. The result is Critical `0`, Important `0`, Moderate `0`; the source is ready for dynamic validation, not for return or closeout.
- The three original REDs are closed in source: expanded post-effect sends propagate target/local parent gain and mute through each downstream hop; master/global gain is composed exactly once on the Kira main track; active graph resync updates both track-send and send-volume handles. Send tracks are staged before track installation, and allocation/validation failures leave the installed graph unchanged.
- The six source/test SHA-256 values remain `40e9de7f9bbd724c1e1045559815510ead4f356c22a9bcad6f1997744c0ee983`, `2bc52f097a2b403237090205625a696ede4efc92dccbc25c7ef6fb9aafb8c33c`, `2bb205b80c11589d61c9cbc572c733504cf1d7cf50885ebcade84c3acaccaafd`, `f6aa449580a723178e5cc8ffe22f1b33c9854b43e984006bc6b404683e5f1aca`, `72d900943fe1da5ed4873c20d7fe756d0be4eabcc3f76ab3de3b4e8b795f1f36`, and `a2a4f6df9d8efcff1d0dabfc52e8b589175beb47f5ee4c1b167aa627c061f19e`. They match the code/test blobs sealed by static manifest `6019fd81698fefe237bce1eeab03f7f6771a32beea2e1e7fa4bc145e62520e1e`; the failure document itself has since advanced, so the old seven-path manifest is not a current whole-record snapshot.
- The passed static ticket `ddc81648724b4006a342068fff6a27e4` remains reusable only for that source-contract layer. Current `Cargo.lock` (`8e7d4220753dfd6a02576321d298d5a75eccdcbcf2bbc2f4d36c3694319fad8e`) and `zircon_plugins/Cargo.lock` (`35033c3ed8892a880f38be02ef0c58faece418d1c9d1aa7b228756e04dbb5c1f`) are modified, and `E:\\Git\\zr_vm` still has foreign tracked and untracked changes. No historical focused, plugin-broad, or upward Render01 Cargo result is current-source/current-configuration evidence.
- Remaining gates are fresh managed focused Cargo, plugin-broad Cargo, upward Render01 acceptance, canonical return records, and coordinator closeout. This failure stays `open` and the primary Session stays `waiting_validation` until those gates are terminal and reusable.

### 2026-09-26 successor r3 current-source intake

The rolling repair resumed under the stable coordinator Session
`failure-roll-01a084c8-plugins02-kira-routing-r3` (request `980530eb5fff4d91a8a53123091eab0c`,
created at `2026-09-26T18:38:44Z`) after the archived r2 owner.  The coordinator
admitted the Session at baseline epoch `612` with base HEAD
`982f8a70b33c6cdbb8b2b62a4e73dca07c530223`; the failure document was claimed by
lease request `8a6eb26b3fe843e69b8fb42def48614b` before this append.  Its pre-append
SHA-256 was `629b32796f94f8ee265b2a9569313430016575850686519faa58351761e4cadc`.

The six current production/test paths are clean and were frozen without edits:

- `zircon_plugins/sound/runtime/src/kira_bridge/graph_compile.rs`
  `40e9de7f9bbd724c1e1045559815510ead4f356c22a9bcad6f1997744c0ee983`
- `zircon_plugins/sound/runtime/src/kira_bridge/graph_compile/routes.rs`
  `2bc52f097a2b403237090205625a696ede4efc92dccbc25c7ef6fb9aafb8c33c`
- `zircon_plugins/sound/runtime/src/kira_bridge/manager.rs`
  `2bb205b80c11589d61c9cbc572c733504cf1d7cf50885ebcade84c3acaccaafd`
- `zircon_plugins/sound/runtime/src/kira_bridge/manager/graph.rs`
  `f6aa449580a723178e5cc8ffe22f1b33c9854b43e984006bc6b404683e5f1aca`
- `zircon_plugins/sound/runtime/src/kira_bridge/manager/graph/transaction.rs`
  `72d900943fe1da5ed4873c20d7fe756d0be4eabcc3f76ab3de3b4e8b795f1f36`
- `zircon_plugins/sound/runtime/src/tests/kira_bridge/graph/routing.rs`
  `a2a4f6df9d8efcff1d0dabfc52e8b589175beb47f5ee4c1b167aa627c061f19e`

The exact focused filter remains
`tests::kira_bridge::graph::routing` (the three original RED names are present
once each).  The previously passed static contract ticket `ddc81648724b4006a342068fff6a27e4`
and manifest `6019fd81698fefe237bce1eeab03f7f6771a32beea2e1e7fa4bc145e62520e1e`
remain reusable only for these unchanged source blobs.  No dynamic result is
claimed: fresh managed focused/plugin-broad Cargo, current lock/configuration,
upward Render01 acceptance, and any required scoped integration are still
pending.  The external `E:\Git\zr_vm` worktree remains foreign-dirty (135
tracked/untracked entries observed), so prior admission rejection
`validation_ticket_external_worktree_dirty` is not treated as a product test
result.  No source files were edited by this Session; the failure remains open
and the Session remains `resolving_failure` until independent review and all
managed gates are refreshed.

### 2026-09-26 r3 static receipt and independent review

- Snapshot `3941` sealed the intake manifest.  The current read-only anchor
  probe executed exactly 20 required anchors across the six source/test files
  and emitted `PLUGINS02_KIRA_ROUTING_STATIC_PASS` (`anchors=20 missing=0`).
  It covered expanded post-effect sends and downstream/local gain and mute
  propagation, send-target gain and topology resync, Kira send-track staging,
  master/global gain composition, transaction staging, and all three original
  RED test names plus the downstream-chain regression.
- Independent reviewer `/root/review_editor03_gizmo_private` audited snapshot
  `3941`; the doc hash and all six source/test hashes matched.  The reviewer
  confirmed the exact filter and static-ticket boundary, did not reuse the
  prior artifact-governance or foreign-worktree admission errors as dynamic
  evidence, and returned Critical `0`, Important `0`, Moderate `0`.
- This is still source/static evidence only.  Fresh managed focused and
  plugin-broad Cargo, current lock/configuration, upward Render01 acceptance,
  canonical `fixed-*` return records, and coordinator closeout remain pending.
