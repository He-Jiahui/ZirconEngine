---
handoff_kind: fixed
status: fixed
created_at: 2026-07-13
resolved_at: 2026-07-14
summary_slug: zrpack-blake3-contract-drift
origin_plan: docs/plans/zircon_plugins/07-net.md
fixing_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
origin_child_dir: docs/plans/zircon_plugins/07
fixing_child_dir: docs/plans/zircon_runtime/runtime/04
related_code:
  - zircon_runtime/src/asset/pack/dedup.rs
  - zircon_runtime/src/asset/pack/manifest.rs
  - zircon_plugins/net/features/content_download/runtime/src/manager/hash.rs
  - zircon_plugins/net/features/content_download/runtime/src/manager/http_fetch.rs
  - zircon_plugins/net/features/content_download/runtime/src/manager/progress.rs
tests:
  - E:/cargo-targets/zircon-engine/pool/0ab59ce30aa63b5c52717a92c1e2e1341f595b8959f221b5793a88271a9c4a4c/debug/deps/zircon_runtime-f8774ee8510e12dc.exe asset::pack::dedup::tests::zrpack_content_hash_matches_blake3_empty_input_vector --exact --nocapture --test-threads=1
  - E:/cargo-targets/zircon-engine/pool/0ab59ce30aa63b5c52717a92c1e2e1341f595b8959f221b5793a88271a9c4a4c/debug/deps/zircon_runtime-f8774ee8510e12dc.exe asset::tests::pack:: --test-threads=1
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_content_download_runtime --locked --jobs 1
---


# Runtime 04：ZrPack BLAKE3 契约漂移

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/07-net.md`
- 来源执行切片：M6-T2 Content Download 断点续传与哈希校验复验
- 修复责任计划：`docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md`
- 交接原因：ZrPack manifest、chunk hash 生成和 pack reader/writer 的最低共享 owner 位于 `zircon_runtime/src/asset/pack`；Plugins 07 只能消费该契约，不能在下载插件内另立真实来源。

## 失败现象与复现证据

Plugins 07 §3.7 与 M6-T2 要求 `ZrChunkEntry.hash: [u8; 32]` 表示 BLAKE3，并要求下载后的 chunk 以同一 BLAKE3 契约校验失败后重拉。当前共享 `ZrPackManifest` / `ZrChunkEntry` 已位于 `zircon_runtime/src/asset/pack/manifest.rs`，但 `zrpack_content_hash(...)` 仍由 `ZRPACK_HASH_SEEDS` 和四次 `fnv1a64(...)` 拼成 32 字节值；这不是 BLAKE3。

同时，`zircon_plugin_net_content_download_runtime` 仍在 `manager/hash.rs` 用 ring SHA-256 生成十六进制字符串，`http_fetch.rs` 与 `progress.rs` 也以 `NetDownloadChunk.sha256: String` 为完成条件。现有 `corrupt_chunk_refetched` 只证明旧 SHA-256 路径会重拉，不能作为 M6-T2 的 BLAKE3 验收证据。

复现：

```powershell
Select-String -Path zircon_runtime/src/asset/pack/dedup.rs -Pattern 'ZRPACK_HASH_SEEDS|fnv1a64'
Select-String -Path zircon_plugins/net/features/content_download/runtime/src/manager/hash.rs -Pattern 'SHA256|sha256_hex'
```

当前两条命令分别命中共享 FNV-1a 拼接实现与插件本地 SHA-256 实现，证明发行侧 ZrPack、下载侧验证与计划约定存在三套不一致语义。

## 最低共享层根因

`ZrChunkEntry.hash` 的 32 字节形状已定稿，但共享 pack owner 没有把算法语义硬切到 BLAKE3；下载插件随后以独立 SHA-256 字符串 DTO 补齐校验，形成重复真实来源。由于 pack writer、reader、delta、dedup 和下载器必须对同一 chunk hash 达成字节级一致，根因属于 Runtime 04 的 asset-pack 契约，而不是单个 HTTP 调用点。

## 架构修复验收

- `zircon_runtime::asset::pack::zrpack_content_hash(...)` 使用 BLAKE3，并以官方已知向量锁定算法；writer、reader、delta 与 dedup 全部继续消费这一唯一函数。
- Content Download 删除本地 SHA-256 成功路径，直接消费共享 `[u8; 32]` ZrPack chunk hash；不得保留双算法、字符串兼容字段或静默回退。
- 下层 ZrPack manifest/writer/reader/delta 测试通过，并新增下载侧 `ZrPackManifest -> range request -> BLAKE3 verify -> corrupt refetch` 集成覆盖。
- 重新运行 Plugins 07 M6-T1/M6-T2 验收，`interrupted_download_resumes_from_bitmap` 与 `corrupt_chunk_refetched` 必须在共享 BLAKE3 契约上通过。

## 禁止临时方案

- 禁止在下载插件内复制 BLAKE3 实现或保留 SHA-256/FNV-1a 兼容分支。
- 禁止新增 hash-algorithm 猜测、按字符串长度自动选择算法、manifest 别名或测试专用 bypass。
- 禁止弱化 Plugins 07 的 BLAKE3 验收文字，或把旧 SHA-256 测试通过记为 M6-T2 完成。

## 修复结果与回传

- 根因：ZrPack shared hash used four concatenated FNV-1a lanes while Content Download independently used SHA-256 strings, creating three incompatible chunk-hash truths
- 架构修复：hard-cut zrpack_content_hash to BLAKE3; replace NetDownloadChunk sha256 string with raw content_hash bytes; delete plugin-local hash owner and ring dependency; verify downloads through the shared Runtime function and real ZrPack manifest entries
- 验证：Runtime BLAKE3 vector 1/1, ZrPack pack filters 43/43, net download contract 1/1; content download package 15/15 including interrupted resume and corrupt refetch; retired FNV/SHA owner scans clear
- 回传：Plugins07 M6-T1/M6-T2 now consume the Runtime-owned ZrPack BLAKE3 contract with no compatibility algorithm or duplicate hash owner
