# A2 Engine bootstrap/layout parity 证据

- Gate: design-ready
- Owner session(s): `root-designment02-penpot-zui-roundtrip-20260831`
- Changed scope: CLI 生成的 `.zui` fixture、`UiZuiAssetLoader` + `UiV2DocumentCompiler` integration contract
- Manifest: [a2-engine-bootstrap.yaml](../manifests/a2-engine-bootstrap.yaml)
- Commands actually run: CLI `project`、`reconcile`、`roundtrip`；`rustfmt --edition 2021 --check`；managed validator dry-run；managed validator 实际测试尝试
- Result summary: 三条 CLI 路径成功；生成 fixture 包含 view root、flex layout、事件、未知 metadata、可达 virtual-row prototype 和 detached node；Rust test 已落盘并通过 rustfmt 检查
- Repaired failures: repeat 从错误的 detached self-prototype 改为 owner 上的 direct-child prototype，符合 runtime virtual-list contract
- Deferred external checks: 共享 Cargo lane 被另一 Session 占用；第一次 validator 在 `cargo.acquire` 回执阶段出现 `command_post_timeout`，第二次明确返回 `cargo_cpu_lane_reserved`，尚未得到 Rust test 终态
- Evidence links: [Rust contract test](../../../../zircon_runtime/tests/zui_penpot_bridge_contract.rs)、[generated fixture](../../../../zircon_runtime/tests/fixtures/ui/penpot_roundtrip.zui)
- Unlocks: Rust test 通过后可把 A2 从 in_progress 提升为 validated；真实 rendered layout parity 仍属于后续 Editor owner 产品证据

当前证据只能证明同一 `.zui` contract 已进入引擎 loader/compiler 验证路径，不能声称像素 parity 或 Editor 产品自举已完成。
