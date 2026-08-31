---
handoff_kind: fixed
status: fixed
created_at: 2026-08-26
resolved_at: 2026-08-28
summary_slug: frameworks01-zr-resource-asset-uri-consumer-attribution
origin_plan: docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
fixing_plan: docs/plans/optimize/zircon_runtime/25-filesystem-path-uri-vfs-mount-watch-sandbox-atomic-io-review.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/01
fixing_child_dir: docs/plans/optimize/zircon_runtime/25
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/asset/watch/asset_uri_for_path.rs
  - zircon_runtime/src/asset/watch/asset_uri_for_path/tests.rs
tests:
  - .\tools\zircon-session.ps1 ownership matrix --prefix zircon_runtime/src/asset/watch/asset_uri_for_path -Json
---

# Runtime25: return executable attribution for Resource AssetUri consumers

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md`
- 来源执行切片：M1 `zr_resource` current-source atomic scope rotation
- 修复责任计划：`docs/plans/optimize/zircon_runtime/25-filesystem-path-uri-vfs-mount-watch-sandbox-atomic-io-review.md`
- 交接原因：Runtime25 owns the active single-pass AssetUri projection Session and its two exact
  consumer attributions.

## 失败现象与复现证据

The stable 503-file Resource consumer union contains two dirty Runtime25 paths. Session
`root-runtime25-single-pass-asset-uri-20260826` remains executable, so Frameworks01 transfer is
blocked by `source_owner_executable` despite zero overlapping live leases. Current hashes are:

- `asset_uri_for_path.rs`: `6ac2ee259b667b5a4d495748509ddbb037c029d34d1c20b34dd0df1055201e47`
- `asset_uri_for_path/tests.rs`:
  `38232ce20e8bdcfd3be90bfaf983f69f916b3e79b72eababd5636fd56dd5d9ad`

Both stored attribution hashes differ from current bytes. Exact evidence is retained at
`F:/zircon-profiles/frameworks01-zr-resource-preflight-20260826/ownership-intersection-b41b0c0b.json`.

## 最低共享层根因

The lowest proven failure is the executable Runtime25 attribution lifecycle. Frameworks01 does not
own the AssetUri optimization semantics and cannot terminalize or return its foreign Session.

## 架构修复验收

- Preserve and reconcile both current blobs, then terminalize or return their Runtime25 ownership
  through the coordinator.
- A fresh two-path ownership-transfer preview must match current hashes and contain no
  `source_owner_executable` reason.
- Frameworks01 must include both files in its refreshed atomic consumer manifest; no consumer
  subset is accepted.

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses,
  or call-site exceptions.
- Do not rewrite AssetUri behavior for attribution cleanup or weaken the Resource hard-cut guard.

## 修复结果与回传

- 根因：Runtime25 source Session expired to stale; its prior executable attribution gate no longer represented a live owner.
- 架构修复：Transferred the two unchanged current AssetUri consumer blobs atomically to Frameworks01 using coordinator fingerprint ecccb84f96c9b674b42377e1534f6bdb62c05cd08aa4a7e33b1e995965f8204d; no behavior or facade change.
- 验证：Post-apply ownership matrices match current hashes 6ac2ee259b667b5a4d495748509ddbb037c029d34d1c20b34dd0df1055201e47 and 38232ce20e8bdcfd3be90bfaf983f69f916b3e79b72eababd5636fd56dd5d9ad with live Frameworks01 leases, zero blocking reasons, and integration_ready state.
- 回传：Runtime25 AssetUri consumer attribution blocker fixed and returned; full zr_resource hard cut remains blocked by eight MVP00 consumers.
