---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: asset-migration-streaming-transaction-journal
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/asset/migration/transaction.rs
  - zircon_runtime/src/asset/migration/transaction/journal.rs
  - zircon_runtime/src/asset/migration/transaction/stage.rs
  - zircon_runtime/src/asset/migration/transaction/commit.rs
  - zircon_runtime/src/asset/migration/transaction/recovery.rs
tests:
  - cargo test -p zircon_runtime --lib asset::tests::migration::project_commandlet::transaction_recovery --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo test -p zircon_runtime --lib asset::tests::migration::project_commandlet::crash_windows::append_journal_records_each_document_once_and_recovers_after_commit_interruption --locked --jobs 1 -- --exact --nocapture --test-threads=1
  - document count, file size and every crash/rollback/recovery window matrices
---

# Runtime04：asset migration streaming transaction journal缺失

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：Runtime asset migration transaction逐Rust文件性能审查，PERF-MVP-512
- 修复责任计划：`docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md`
- 交接原因：durable schema、live replace与recovery authority必须由同一transaction owner原子升级，不能在调用点绕过fsync。
- 生命周期键：`asset-migration-streaming-transaction-journal`

## 失败现象与复现证据

intent和stage分别整读hash原target，stage另copy backup；commit整读staging再由`atomic_write`写第二临时文件，rollback整读backup。每个document commit前后都clone全部D条state、pretty TOML、atomic write并fsync完整journal，累计O(D²) journal bytes。recovery对多类artifact先整读hash，再整读UTF-8/TOML验证。

## 最低共享层根因

journal schema只有全量snapshot，没有immutable intent + compact per-document transition；file helpers只接受完整byte slice，没有streaming hash/copy或已同步staged-file atomic replace边界。

## 架构修复验收

- 新schema用immutable intent manifest和append/fixed-slot state records；每次live write前持久化单document transition，recovery fold为唯一state，journal written bytes O(D)。
- stage写入时计算new digest，backup streaming copy同时hash；commit直接atomic replace已sync staged artifact，不整读payload或再造payload temp。
- recovery一次stream完成digest与TOML evidence，保持untrusted journal、allowed target、link/reparse、role和identity验证。
- Runtime11只并行独立stage/hash；live commit与危险窗口fsync保持ordered serial，cancel仅发生于安全阶段。
- documents 1/1k/100k、file 4KiB/1GiB及全部fault windows记录journal bytes/fsync、file read/write/copy、RSS：journal O(D)、额外内存bounded、commit payload copy=0；现有crash/rollback/malicious/idempotence/Windows durability全通过。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止删除或延后危险live write前的durability barrier来换性能。
- 禁止把完整payload Vec包进Arc冒充streaming I/O。

## 修复结果与回传

## 当前实施进度

- 2026-07-27：已将 transaction journal hard cut 到 v3 immutable intent 加 append-only transition records；每个文档只追加 prepared、committing、committed 状态，恢复先折叠该状态机再验证。
- stage 现在在写入时计算新文件摘要，backup/retired backup 以固定大小 buffer 流式复制并同步；commit 与 rollback 直接原子替换已同步的 staging/backup 文件，不再整读 payload 后二次写入。
- recovery 对每个已出现 artifact 缓存一次流式摘要和 TOML 证据，继续验证 target allowlist、路径身份、sibling role 以及 link/reparse 边界；既有 crash/rollback 窗口已在实现中保留。
- rollback 在每个 restore 前追加 `rolling_back` transition；恢复仅在该状态接受可证明的原始或新 target，覆盖 target restore 与 retired sidecar restore 之间的中断窗口。
- 已添加三文档 commit interruption regression，要求 immutable document records 固定且 transition records 线性增长，并在下一次 Apply 收敛。
- 静态检查：`rustfmt +1.94.1 --check --edition 2024` 通过；`git diff --check` 通过（仅工作树 CRLF 提示）。
- 静态审查在首个受管预留启动前发现初始 journal 会序列化空 `transitions = []`，与后续 `[[transitions]]` 追加冲突；已将该字段改为序列化时跳过空数组、反序列化时保持默认。旧的未启动 reservation `e83de09abe8a4c34bf5c80015c7d0147` 已按本会话所有权释放，未产生 Cargo job。
- 受管 Cargo 验证已重新排队：reservation `cb93c33ee7d042b0b06b3ceca931cf30`，source manifest fingerprint `1ba8d91c4dabfa00a385e9cc08ea77eacefad8c033aef374105a19310fddb5e6`，命令 `cargo +1.94.1 test -p zircon_runtime --lib asset::tests::migration::project_commandlet::transaction_recovery --locked --jobs 1 -- --nocapture --test-threads=1`。尚未产生终态，不声明通过。
- 2026-07-26 current-source gate 已自然终态：job `c43e14bc8a35459eaca0a04bfb65f6c3` / run `5e9cbb9d4f77400abefad23e56a4e464`，exit `101`，运行 35m16s 后在 lib-test compile 终止，目标 `transaction_recovery` 未执行。原始 stderr 的唯一 error 是本切片 `transaction/commit.rs:107` 对 `record_document_state` 的未导入调用（E0425）；Runtime15 路径只产生 warnings，不能误归为根因。
- 已补 `commit.rs` 对私有 journal transition writer 的直接导入；上述 reservation/job 不可复用，修复后的 source manifest 必须作为新 FIFO 受管验证重新申请。当前记录继续保持 open，不声明行为测试通过。
- 2026-07-26 retry 的 current-source terminal：job `491a6b45db7f4189a0a7780b6747341f` / run `cd356d30028d417f88c82b0575a363da`，exit `101`，运行 31m20s 后在 lib-test compile 终止。此前的 transaction E0425 已不再出现；新的 13 个 E0451 均为 `native_plugin_loader::NativePluginLoadReport.projection` 私有字段与 `load_discovered` / `native_plugin_live_host` 测试的 struct update 构造不一致。目标 `transaction_recovery` 仍未执行，不构成 transaction RED 或 GREEN。
- 该 blocker 的活动 owner 是 Frameworks04 session `frameworks04-native-load-report-projection-compile-boundary-r2-20260727`，已持有相关 `native_plugin_loader` 路径租约；Runtime04 不修改这些外部路径。等待 owner 修复并产生可编译 current source 后，Runtime04 将以新 source-bound FIFO gate 重试。
- append journal regression 的 exact 命令已列入本记录；将在第一条 focused compile gate 终态后按 FIFO 申请，避免把两条独立测试混为同一证据。
- 2026-07-28：recovery artifact evidence 已改为固定 64 KiB 缓冲的一次流式 BLAKE3 摘要与 TOML 结构证据，不再把整个 artifact 累积到 `Vec` 后再 UTF-8/TOML 检查；artifact 路径在单次 recovery 中仍只读取一次。新回归覆盖逐字节分块、跨块 UTF-8、空/单/多行字符串与转义分隔符、未闭合容器、伪造非 TOML 备份以及 `toml::to_string_pretty` 的规范化迁移输出。
- 该实现的 `rustfmt +1.94.1 --check --edition 2024` 与 `git diff --check` 已通过。两次 `validate-matrix` 聚焦申请均由协调器拒绝启动，先后因为 Shader06 与 Runtime07 已保留下一条 CPU lane；没有未受管 Cargo 进程，也尚未产生 Runtime04 Cargo 终态。

Open state: `实现中，待受管当前源验证`; no pass is claimed.
