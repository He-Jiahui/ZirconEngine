---
handoff_kind: fixed
status: fixed
created_at: 2026-07-17
summary_slug: source-manifest-build-config-cap
origin_plan: docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_runtime/render/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/cargo_jobs.py
tests:
  - tools/session_coordinator/tests/test_cargo_reservations.py::CargoReservationTests::test_cpu_reservation_supports_first_class_large_source_manifest_and_rechecks_all_entries
resolved_at: 2026-07-17
---


# Coordinator01：源码清单被 build_config 长度上限截断

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md`
- 来源执行者：`render01-compiled-pipeline-source-validation-20260717`
- 来源执行切片：compiled-pipeline current-source source-bound validation
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Cargo 预约兼容性、源码绑定与 reservation→consume→start 原子性均由 Coordinator01 的 `CargoJobService` 负责；Render01 无权缩减自己拥有的架构路径或绕过调度器。

## 失败现象与复现证据

- 旧协议把 `path=SHA256` 的 `source_manifest` 塞入 `CargoCompatibility.build_config`；该单一文本字段被限制为 4096 个字符。
- Runtime12 的 27 路径清单约 3550 字符，能够通过；Render01 的完整清单必须包含租约目录 `zircon_runtime/src/graphics/pipeline/declarations/compiled_render_pipeline` 的四个递归 Rust 子文件，合计 68 路径，不能通过该字段上限。
- Render01 当前唯一正确的顺序化 `path_key=lowercase_sha256` 指纹为 `4322914a3cb0f9709987f9745d1d7cba5b327fc3a0b378fe531e6e29d9213368`。先前 64 路径 `3744b95e…` 漏掉四个已拥有目录子文件；早期完整 68 路径 `270fbc0b…` 又在 final frame-scan 修复后过期，二者均明确失效。

## 最低共享层根因

`build_config` 同时承担小型构建配置和大规模源码身份清单，且 `CargoCompatibility.canonical()` 对所有字段应用同一个 4096 字符限制。清单不是构建配置；由此把完整架构范围误判成无效兼容性，而 reservation/start 也没有独立记录可审计的清单大小与指纹。

## 架构修复验收

- 将 `source_manifest` 作为兼容性 JSON 的一等字段持久化，独立验证路径、SHA-256、最大条目/字节量及稳定指纹；`build_config` 继续仅保存构建配置。
- 兼容旧的 `build_config.source_manifest` 读取，但新预约使用一等字段；reservation 创建与 `cargo run-reserved` 启动都必须验证同一完整清单。
- 目录租约不得以目录名替代源码：调用方必须提交递归展开后的显式 Rust 文件表；任一表项在 reservation 后漂移、缺失或新旧 payload 不一致均拒绝启动。
- 覆盖超过 4096 字符的 68 项清单：reservation 成功、返回独立指纹；consume 后改变任意一项文件，启动以 `source_manifest_stale` 拒绝。
- Render01 只在 Runtime12 fixed SHA 之后，原子递归展开目录 lease、重新哈希当前 68 项源并确认 `4322914a…` 后创建新的预约；不得创建或运行 64 项、`270fbc0b…` 或任何过期 payload。

## 禁止临时方案

- 不得删减目录租约的递归 Rust 文件、把目录路径伪装为单个文件，或使用部分清单。
- 不得提高 `build_config` 的无界文本限制来混合两类数据，或绕过 reservation/start 双重哈希校验。
- 不得重启、抢占或中断当前真实 Performance Cargo；待自然空窗后再用正常 rollover 加载修复。

## 修复结果与回传

- 根因：The first-class source_manifest payload was still capped at 1024 entries, rejecting the complete 1275-file Sound hard-cutover scope before any reservation or Cargo process was created.
- 架构修复：Raised the independent payload limit to 4096 entries while retaining the 256 KiB byte bound, normalized path-to-SHA256 contract, and reserve plus run-reserved rechecks; build_config remains separate and partial manifests remain rejected.
- 验证：Red 1275-entry reservation reproduced the 1024-entry rejection; the repaired 1275-entry reserve-consume-final-entry-drift regression and 4097-entry bound rejection passed; cargo reservation suite passed 33/33; py_compile and scoped git diff check passed; admission-preserving rollovers 9a67784b31724172afd7a42b546bd429 and f007986fe37849bea306e3d2a0250588 loaded the payload and child-record return contracts.
- 回传：Coordinator01 returned the source-manifest capacity contract to Render01 without taking its directory lease. Sound focused validation has an independent raw compiler failure; any later broad gate must recompute the complete current manifest and cannot use a partial payload.
