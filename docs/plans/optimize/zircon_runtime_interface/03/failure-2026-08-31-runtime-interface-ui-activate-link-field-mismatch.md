---
handoff_kind: failure
status: open
created_at: 2026-08-31
origin_plan: docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
fixing_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/01
fixing_child_dir: docs/plans/optimize/zircon_runtime_interface/03
summary_slug: runtime-interface-ui-activate-link-field-mismatch
observed_at: 2026-08-31
related_code:
  - zircon_runtime_interface/src/runtime_api/host/ui_host_request.rs
  - zircon_runtime_interface/src/ui/dispatch/input/result.rs
tests:
  - cargo test -p zircon_runtime_interface --locked
  - cargo test -p zr_resource --locked --release --lib resource_management_projection_current_source_profile -- --ignored --test-threads=1 --nocapture
---

# Runtime Interface UI Activate-Link Field Mismatch

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md`
- 来源执行切片：R14 current-source ResourceManagement release profile
- 修复责任计划：`docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md`
- 交接原因：最低共享原因位于 RuntimeInterface typed UI dispatch 到 generic host request 的公共投影，不属于 Frameworks01 或 `zr_resource`。

## 失败现象与复现证据

Frameworks01 managed current-source ResourceManagement release profiling reached the
`zircon_runtime_interface` dependency and failed before compiling `zr_resource`:

- Cargo job: `64107e1527764083b78888c199cccf5f`;
- run: `4630a785c5304a90957462c8f04c6581`;
- profile command: `cargo test -p zr_resource --locked --release --lib
  resource_management_projection_current_source_profile -- --ignored --test-threads=1
  --nocapture`;
- compiler: Rust 1.94.1 Windows MSVC managed lane;
- `zircon_runtime_interface/src/runtime_api/host/ui_host_request.rs:139`: E0026/E0027.

`UiDispatchHostRequestKind::ActivateLink` now has fields `target` and `link_target`, with
`link_target` serialized as `href`. The runtime host-request projection still matches the Rust field
as `href`:

```text
UiDispatchHostRequestKind::ActivateLink { target, href }
```

The pattern names a removed field and omits `link_target`, so the support crate does not compile.

## Exact current source

| File | State | SHA-256 |
| --- | --- | --- |
| `zircon_runtime_interface/src/runtime_api/host/ui_host_request.rs` | untracked, coordinator `attribution_missing` | `035671d851407158cffbfc8054e9b623c68ccb1221b4eaa840cdc00d6f329068` |
| `zircon_runtime_interface/src/ui/dispatch/input/result.rs` | modified, archived attribution stale | `d94bfb575396554c15221b6bd88845857679b6a896ba5be4adf56d3818aa8278` |

Coordinator evidence:

- bridge ownership matrix request `c7718b340a604ba9a03aea994939f152`: unowned,
  `attribution_missing`;
- dispatch definition matrix request `0b8dff12fad34de7a68080d0f946f442`: archived source owner,
  `attribution_hash_stale`, `attribution_baseline_stale`, `owner_not_executable`, and
  `live_lease_missing`.

No existing `failure-*.md` matched this source/error before this record was created.

## 最低共享层根因

Frameworks01 did not claim or edit either Runtime Interface source. The fixing owner must first
resolve whether the new runtime host-request bridge and the UI dispatch hard cut belong to the Runtime
Interface host ABI plan or the UI dispatch/link plan, then claim/attribute the exact files through the
coordinator. Frameworks01 must not absorb an untracked mixed-era bridge merely to unblock its profile.

## 架构修复验收

The fixing owner must:

1. preserve the Rust field hard cut to `link_target`; the serialized external key may remain `href`
   through the existing serde rename;
2. update every typed projection, constructor, pattern, and test consistently, without a legacy
   `href` Rust field or compatibility variant;
3. validate `zircon_runtime_interface` on Windows Rust 1.94.1 with `--locked`;
4. claim and attribute the current files, including the untracked bridge, before coordinator commit;
5. return the commit/current hashes and a fixed artifact or canonical return to this Failure.

## 禁止临时方案

- 不得恢复 Rust `href: String` 字段、兼容 variant、字符串 fallback 或第二份 link parser。
- 不得 wildcard/suppress E0026/E0027、弱化 typed projection test，或让 Frameworks01 吸收 foreign UI source。
- 不得在 managed lower/upward gates 完成前把 source repair 标成 fixed 或 accepted。

Frameworks01 can then rerun the unchanged 31-sample ResourceManagement profile. This failure blocks
that managed profile only; it does not block readiness architecture review or behavior/profile
infrastructure work.

## 修复结果与回传

RuntimeInterface03 owns the typed UI dispatch/host projection. The lowest shared cause was a
mixed-era bridge: dispatch had already hard-cut `ActivateLink` to the admission-checked
`UiRichLinkTarget`, while `runtime_api/host/ui_host_request.rs` recreated the removed Rust field
`href: String` and attempted to destructure it from the typed dispatch request.

The current-source repair keeps one typed model end to end:

- `ZrRuntimeUiHostRequestKindV1::ActivateLink` now owns
  `link_target: UiRichLinkTarget`;
- both dispatch and host bridge serialize that Rust field through the existing external key
  `href`; no Rust `href` field, compatibility variant, string fallback, or second parser was added;
- the bridge projection clones the already-admitted `UiRichLinkTarget` and its focused regression
  locks typed identity, `href` wire shape, round trip, and content-free `Debug` output;
- current bridge SHA-256:
  `c977c4d6689fb2487a9d7a179addf6977450aefdebff96d64be7c59bc1707b51`;
- current dispatch-result SHA-256:
  `b85724d756c6140a3d3113d052d163b6b9cdeac5c2a3525d5ba66805f4be4a15`;
- current dispatch-effect SHA-256:
  `4d5c62c9c64f42e8334fa91f918e5994d077f5f59da885f8114a79e03acaa006`;
- scoped `rustfmt`, `git diff --check`, and an all-Rust-source legacy typed-field scan completed;
  `legacy_typed_href_matches=0`.

The three-file typed chain now has one legal RuntimeInterface03 owner. Ownership transfer request
`3e3565f879c6464087a21f297594fd11` moved the bridge, dispatch result, and dispatch effect into
`root-runtime-interface03-activate-link-failure-20260831`; lease request
`92ca5c1340ba42d1be88c85c9d91d20c` acquired all three source paths plus this Failure, and
attribution request `0173823781624976b2520df1cca4385e` recorded their current hashes.

The source later converged with the borrowed pointer-route receipt fix without changing the
ActivateLink projection. The exact RuntimeInterface03 lease was renewed by request
`d26b2cbfc8f6467da5e2f6630f6877bd`. One focused static batch covering the input-routing receipt,
borrowed dispatch-route sharing, pointer dispatch ownership, and typed rich-link chain completed
`46/46` GREEN. The all-Rust typed-field scan still reports `legacy_typed_href_matches=0`; `href`
appears only as the stable serde wire key and in assertions that reject a `link_target` wire key.

The first managed lower and upward tickets did not execute Cargo. Tickets
`744cfd6ce9474f5fad6ebffa354dd278` and `5e867b1dd1b94787804fa08af4037707` both failed during
materialization with `validation_copy_overlay_not_owned`; the overlay named this Failure,
`ui_host_request.rs`, and `result.rs`. The ownership transfer above closes that control-plane cause,
but the current bridge hash also advanced after scoped formatting, so neither failed ticket is
current-source validation evidence.

Status remains `open`: a new exact-manifest Windows Rust 1.94.1 `--locked` lower-layer gate and the
originating Frameworks01 profile rerun are still required. No `fixed-*` return or milestone commit
is valid before those gates execute and pass.

The current-source lower-layer gate is now queued as a shared RuntimeInterface03 batch:

- ticket `cd1d6cff000c42d18bf7d4a3abb4be84`;
- submit request `0ffcd0e158c34e8b91dcf6e2b918bff7`;
- command `cargo +1.94.1 test -p zircon_runtime_interface --locked --release --jobs 1`;
- source manifest captured the current bridge, dispatch result/effect, pointer route, and all three
  canonical failure records.

This record remains open until the managed terminal receipt and the originating `zr_resource`
profile rerun are both available.
