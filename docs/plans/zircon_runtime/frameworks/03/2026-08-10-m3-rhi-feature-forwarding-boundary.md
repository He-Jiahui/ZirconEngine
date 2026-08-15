---
related_code:
  - Cargo.toml
  - zircon_runtime/Cargo.toml
  - zircon_runtime/runtime-feature-presets.toml
  - zircon_runtime/crates/zr_rhi/Cargo.toml
  - zircon_runtime/crates/zr_rhi_wgpu/Cargo.toml
  - zircon_app/Cargo.toml
  - tools/tests/test_frameworks_03_server_feature_boundary.py
  - tools/tests/test_frameworks_03_profile_feature_presets.py
plan_sources:
  - docs/plans/zircon_runtime/frameworks/03-optional-features-and-profile-matrix.md
reference_engines:
  - dev/bevy/Cargo.toml
tests:
  - python -B -m unittest tools.tests.test_frameworks_03_server_feature_boundary -v
  - python -B -m unittest tools.tests.test_frameworks_03_profile_feature_presets -v
---

# Frameworks03 M3 RHI feature forwarding boundary

Plan: docs/plans/zircon_runtime/frameworks/03-optional-features-and-profile-matrix.md

Milestone: M3 first physical-member slice

Status: source_ready_secondary_review_green_managed_validation_pending

Files: ["Cargo.toml","docs/plans/zircon_runtime/frameworks/03-optional-features-and-profile-matrix.md","docs/plans/zircon_runtime/frameworks/03/2026-08-10-m3-rhi-feature-forwarding-boundary.md","tools/tests/test_frameworks_03_server_feature_boundary.py","zircon_runtime/runtime-feature-presets.toml"]

## Scope Delivered

- Set both workspace-owned RHI dependencies to `default-features = false`. Runtime remains the
  only feature-composition owner: `graphics` activates `zr_rhi_wgpu`, while `platform-winit`
  forwards the platform hook without allowing member defaults to grow the package graph.
- Corrected the static local-package closure algorithm so the first request for a non-optional
  path dependency is visited even when it requests no feature and has defaults disabled. The old
  helper silently omitted exactly the neutral package shape used by `zr_rhi`.
- Added a Server closure contract for both Runtime and App. The closure must contain neutral
  `zr_rhi`, exclude `zr_rhi_wgpu`, and exclude backend/platform member features.
- Added the matching positive contract for Client and Editor: both profiles must activate
  `zr_rhi_wgpu` and forward `platform-winit` to the neutral and backend members.
- Forward-fixed a current cross-plan profile drift found by the full Frameworks03 suite: the
  canonical Editor and Dev `app_features` now include the already-wired
  `first-party-neural-editor-plugin`. The schema v2 TOML remains the single profile source; no App
  compatibility feature, fallback, or hand-written Rust preset was introduced.

## TDD And Static Evidence

- The new focused test first failed because the workspace RHI dependencies implicitly enabled
  defaults. After that repair it exposed the lower closure bug by reporting neutral `zr_rhi` as
  unreachable; the first-request traversal repair closed that false negative.
- Focused RHI closure: 1/1 GREEN. Complete server feature boundary: 14/14 GREEN.
- The first full five-owner run was 54/55 and identified only the Neural Editor/Dev canonical
  profile mismatch. After the forward fix, the same five-owner Frameworks03 suite is 55/55 GREEN.
- Root/profile TOML parsing and Python AST parsing are GREEN. No local Cargo command was run or is
  claimed as acceptance evidence.
- Independent immutable second review is C0/I0/M0. Its exact5 pre/post fingerprint is
  `f532c261c6715c27bf4a55bc35c53e1e0a7acfb9050d6af4767db3aabfd68a83`; the review reran the
  server boundary 14/14, profile preset parity 11/11, AST/TOML parsing, and scoped diff check.

## Remaining Acceptance

- Coordinator receipt `0c6bfaa1adbc4f77977298e8375f5de0` seals 11 inputs for the focused
  Runtime/App RHI closure and canonical profile parity checks. A second durable request,
  `frameworks03-m3-rhi-feature-forwarding-rust-server-v1-20260810`, seals 8,021 current compile
  inputs for canonical Rust 1.94.1 `zircon_runtime --lib --no-default-features --features
  target-server --locked` validation. Both terminal results remain pending coordinator wakeup.
- Queued validation delays accepted closeout only; it does not reopen the completed implementation
  or second review, and this Session does not poll either receipt.
- M3 remains open until every physical `zr_*` member forwards features through the Runtime facade
  and managed build/timings evidence proves Server excludes all optional client/editor members.
