---
related_code:
  - zircon_runtime/Cargo.toml
  - zircon_app/Cargo.toml
  - zircon_runtime/src/core/framework/mod.rs
  - zircon_runtime/src/core/framework/net
  - zircon_runtime/src/core/manager
  - zircon_plugins/net/runtime/Cargo.toml
  - zircon_plugins/net/features
implementation_files:
  - zircon_runtime/Cargo.toml
  - zircon_app/Cargo.toml
  - zircon_runtime/src/core/framework/mod.rs
  - zircon_runtime/src/core/manager/mod.rs
  - zircon_runtime/src/core/manager/resolver.rs
  - zircon_runtime/src/core/manager/service_names.rs
  - zircon_runtime/src/core/manager/tests.rs
  - zircon_plugins/net/runtime/Cargo.toml
  - zircon_plugins/net/features/content_download/runtime/Cargo.toml
  - zircon_plugins/net/features/http/runtime/Cargo.toml
  - zircon_plugins/net/features/reliable_udp/runtime/Cargo.toml
  - zircon_plugins/net/features/replication/runtime/Cargo.toml
  - zircon_plugins/net/features/rpc/runtime/Cargo.toml
  - zircon_plugins/net/features/websocket/runtime/Cargo.toml
  - tools/tests/test_frameworks_03_contract_feature_boundary.py
plan_sources:
  - docs/plans/zircon_runtime/frameworks/03-optional-features-and-profile-matrix.md
  - docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - python -m unittest tools.tests.test_frameworks_03_contract_feature_boundary tools.tests.test_frameworks_03_server_feature_boundary
  - cargo +nightly check -p zircon_runtime --lib --no-default-features --features net-contracts --locked --offline --jobs 1
  - cargo +nightly check -p zircon_runtime --lib --no-default-features --features target-server --locked --offline --jobs 1
  - cargo +nightly check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime -p zircon_plugin_net_content_download_runtime -p zircon_plugin_net_http_runtime -p zircon_plugin_net_reliable_udp_runtime -p zircon_plugin_net_replication_runtime -p zircon_plugin_net_rpc_runtime -p zircon_plugin_net_websocket_runtime --lib --locked --offline --jobs 1
doc_type: acceptance-evidence
status: passed
---

# Frameworks 03 Net contract feature boundary 验收证据

## 范围

本记录覆盖 `net-contracts` 的独立门控与所有直接插件消费者。ZRPack asset owner 前置迁移见 `frameworks-03-zrpack-asset-owner-hard-cutover.md`。Physics/Sound、完整 Runtime/App 测试门和 M2 不在本记录完成范围内。

## 硬切结果

- Runtime/App 暴露同名 `net-contracts`；Client/Editor 预设包含，Server 不隐式包含。
- `core/framework::net` 只随 feature 声明；`core/manager` 的 Net trait、service name、typed holder 和 resolver 同门裁剪。
- base Net、content-download、HTTP、reliable-UDP、replication、RPC、WebSocket 七个直接引用 Net DTO 的 plugin crate 各自在自己的 `zircon_runtime` dependency 上请求 `net-contracts`。
- 不依赖 base Net plugin 的传递 feature 合并，不提供旧 alias、placeholder manager 或兼容 re-export。

## 验证

- Contract/server 静态守卫 12/12 通过；resolver cfg 人工确认绑定 `NetManagerHandle`。
- WSL nightly locked/offline Runtime `net-contracts` 单开通过，8m04s。
- WSL nightly locked/offline Runtime `target-server` 无 Net 通过，3m22s。
- 七个 Net plugin package 的统一 WSL nightly locked/offline check 通过，6m53s。
- scoped nightly rustfmt 与 `git diff --check` 通过。

## 当前判定

Frameworks 03 M1 Net contract 切片完成。M1 仍进行中；下一步进入 Physics contract 的中立资产/场景数据与可选模拟契约边界设计和硬迁移。
