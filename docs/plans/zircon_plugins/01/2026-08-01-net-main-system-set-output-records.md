# Plugins 01 Net standard SystemSet output record

Plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
Milestone: M1 / standard plugin SystemSet rollout
Status: implementation and independent final review completed; managed focused test pending authorization
Date: 2026-08-01

## Recovery Scope

- `plugins07-main-system-set-r1-20260801` retained `active` status but had no live lease for more than six hours. Coordinator ownership preview `295a70826c093e0a1ae5ad80aa7e0e8fbdb4c5565a667a76c5ed3e9be500145f` declared all six exact paths eligible; transfer request `43f4a0bbc29f4a5e83e0eafbb0248a95` and lease request `f4f197e1cbdf4724bc563e09b4cbf42b` completed before source edits.
- The recovery is limited to `net.main` standard-set projection and preserves the existing specialised `net.transport` set. It does not absorb the Net plan's separate transport/executor/runtime review scope.

## Scope Delivered

- Hard-cut ambiguous `NET_SYSTEM_SET` into `NET_MAIN_SYSTEM_SET = "net.main"` and `NET_TRANSPORT_SYSTEM_SET = "net.transport"`; repository search reports zero old-identifier sites.
- Net descriptor and generated `plugin.toml` declare the ordered pair `["net.main", "net.transport"]`.
- Both runtime scene systems join both sets in the same order, preserving their existing First/Last stages.
- Registration and manifest tests now assert production constants and the complete two-set projection.

## Fresh Evidence

- Existing TDD test `net_runtime_systems_join_main_and_transport_sets` was RED before recovery: `net.main` did not exist in descriptor, manifest, or system registrations.
- Rust 1.94.1, edition 2021 scoped `rustfmt --check` and six-file `git diff --check`: passed.
- Static contract check: exact main/transport literals true, descriptor projection true, each registration set count `2`, manifest projection true, old identifier sites `0`.
- `python tools/audit_plugin_structure.py --json --repo-root E:\\Git\\ZirconEngine` no longer reports a Net SystemSet violation. The current manifest-schema failures are unrelated Frameworks04 work: generated headers for `zircon_plugins/native_dynamic_fixture/plugin.toml` and `zircon_plugins/native_window_hosting/plugin.toml`.
- Current source SHA-256: `plugin.toml` = `5F2FE3C0C714D815A941816EBDA792AC906AEA4001699CB37608464F8A561CD4`; `lib.rs` = `FD12540AB053E3D939A58917825318E3C405FE95341C971E1097E62872FE8DBF`; `plugin.rs` = `2F5BF70DAFC0506455902E8752595C002C6A43CBF71BB9C09EE5733E75F8C8FC`; `runtime_system.rs` = `3E887D62A94F1FD337B68A76C12208E0F3263EC0A5A253DF26B897938CBC927B`; registration tests = `D44E5213DE92C8AACC01CA2B5583B8922106A3626833914884903EF4AB5CDFA8`; manifest tests = `3AADE0B63161768E09D49E2C02B9E4EA7DBADDA7475BBEEC0CC9E8743F373349`.
- Fresh independent review: `C0/I0/M0`; it rechecked descriptor/TOML/runtime registration order, owner-filtered membership assertions, scoped rustfmt, diff check, schema suite, and the absence of `NET_SYSTEM_SET`. Cargo was not run.

## 状态和完成项目

| 项目 | 状态 | 证据 |
|---|---|---|
| `net.main` 标准 SystemSet | completed | production constant and recovery TDD contract。 |
| `net.transport` specialised set | completed | explicit replacement constant; no generic compatibility alias remains。 |
| descriptor / TOML projection | completed | ordered two-set declaration on both surfaces。 |
| both runtime system memberships | completed | ingress and egress each join main then transport。 |
| cross-manifest SystemSet guard | completed | Net violation removed from repository audit。 |
| managed focused Cargo test | pending authorization | target: `zircon_plugin_net_runtime::net_runtime_systems_join_main_and_transport_sets`; existing foreign validation-copy remains unavailable to this Session and will not be recreated, retried, or cleaned. |
| independent final review | completed | fresh read-only review returned `C0/I0/M0`; static scope is accepted. |

## Remaining Scope

- Do not use this record to claim Net runtime transport behavior, executor/concurrency, or the full Plugins07 plan as complete.
- When the coordinator explicitly authorizes a managed validation lane, run only the focused Net SystemSet test and append the actual terminal result. The existing foreign validation copy is not recreated, retried, or cleaned by this session.
