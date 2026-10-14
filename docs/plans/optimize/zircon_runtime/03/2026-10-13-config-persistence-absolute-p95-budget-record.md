# Runtime03 配置持久化绝对 P95 预算记录

用户已授权联网研究并由 agent 给定数值。核心决策由 gpt-6.1-sol max 完成，root 核对原始资料及实际 producer 后采用以下工程目标。数值已定义，Native、原始样本和性能验收仍待回执。

| ASCII 负载 | 写入键数 | flush 路径 P95 上限（ms） | 最后 owner 关闭路径 P95 上限（ms） |
|---|---:|---:|---:|
| 1 B | 1 | 50 | 75 |
| 1 B | 1000 | 75 | 100 |
| 1 MiB | 1 | 75 | 100 |
| 1 MiB | 1000 | 100 | 125 |
| 100 MiB | 1 | 1500 | 1600 |
| 100 MiB | 1000 | 1750 | 1850 |

每个上限同时约束两个指标：主指标 `derived_burst_plus_completion_ns = burst_ns + completion_ns`，以及该路径的 `completion_ns`。前者约束已计时的 setter burst 与完成耗时之和，避免只看 flush/Drop 等待而掩盖调用端变慢。它由两段独立计时组成，不包含两段之间的间隔、最初负载构造、burst 前 activation/recovery、后续校验/日志和其他 task-graph 关闭；因此不能称为完整 session 或单个连续 wall-clock 区间。

矩阵仍为 3 种负载 × 2 种键数 × 2 种完成方式 × 5 次，共 60 个样本。大负载是一份 ASCII string，再加 1 或 1000 个数值键，不是 1000 份 100 MiB 负载。每个 5 样本格分别排序，nearest-rank P95 使用零基索引 `ceil(0.95×5)-1 = 4`，即该格最大值。用整数纳秒比较 `limit_ms×1000000`，不合并不同负载、完成方式或环境，不以 worker 最近 attempt P95 代替调用路径 P95。5 个样本只给出本轮观测门，不能证明稳定总体 P95 或统计覆盖率。

推定的设计依据是单 worker pretty encoding 125 MiB/s、local SSD commit path 200 MiB/s：100 MiB 对应约 800 ms + 500 ms，再给 dispatch、map/owner、metadata/sync/replace barrier、burst 与 join 留 200–550 ms。这两个吞吐值是预算设计假设，尚未在本机测得。[serde 官方历史基准](https://github.com/serde-rs/json-benchmark)采用旧版本和旧 CPU，取两次运行中更快的结果，不能证明当前 pretty JSON、文件提交或 P95；[Microsoft FlushFileBuffers](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-flushfilebuffers)解释了缓存和同步写入成本，[Rust sync_all](https://doc.rust-lang.org/std/fs/struct.File.html#method.sync_all)说明同步内容及元数据与关闭文件的成本不同。数值为项目工程目标，外部资料未给出这些上限。

原 2 s shutdown wait 与 30 s flush safety timeout 保持为生命周期控制。原 70% / 75% helper 比例、其他历史 producer、资源/容量门和 45 / 36×3 / 60 矩阵继续保留；新增上限不替代它们。固定 Windows Release/local SSD NTFS runner 的实际 CPU、磁盘、文件系统、toolchain、features、source/config/lock seals 和负载/噪声须由 Native 回执绑定。超限保留全部失败样本并继续优化，不能读取结果后放宽门槛。[NIST quantile](https://www.itl.nist.gov/div898/software/dataplot/refman2/auxillar/quantile.htm)与 [tolerance limits](https://itl.nist.gov/div898/software/dataplot/refman1/auxillar/tolelimi.htm)用于说明估计规则与总体覆盖/置信条件的区别，本计划显式采用上面的 nearest-rank 规则。

机器契约：[runtime03-absolute-p95-criteria.json](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime03-absolute-p95-criteria.json)。原 producer 的物理文件为 `zircon_runtime/src/foundation/runtime/worker/tests/config_original_matrix.rs`，SHA-256 `ee9de4d19642bda6f4572ac99911474fe5da509071ebd45a3200d83c4e0d3038`，literal selector 为 `foundation::runtime::config_manager::worker::config_original_matrix::runtime03_original_config_persistence_payload_key_burst_matrix`；源码、字段、marker、样本规模保持不变，预算作为独立 criteria attachment 使用。

60 行全部 complete、errors 为空、dirty/persisted/pending/failure 和 attempt/write/bytes 守恒、真实 background writer、磁盘 JSON 完整、last owner worker exited、各 session task-graph 真实 join、Native exit0 与编译输入身份匹配之后，才评估 12 格各自的两个 P95。producer 的 `numeric_performance_budget: null` 是原格式；它既不拒绝本独立契约，也不表示达标。每个 request/attempt/source/raw sample 身份保留，不覆盖原始数据。

[batch09 交接保留件](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/batch09-handoff.json)已由 root writer exit0 写入并校验完全一致的 records 副本。其 SHA-256 为 `c934086320a3c4598f8b441b253fdbcf6d029348c7c00b745d5df1977d4a653f`。新增预算将随下一批 Runtime 关闭观察及 Editor08 修复合批交接；当前没有队列、Native 或性能通过证据。

- [x] 用户授权、primary sources、实际 metric 和 12 格上限已记录。
- [x] 原 producer identity、60 样本和历史门保留。
- [ ] 取得绑定实际源与环境的 Native 完整回执和原始样本。
- [ ] 逐格校验两个 P95，并通过原功能、比例、资源与真实 join 门。
- [ ] 完成 Runtime03 产品、Jenkins 和 milestone 验收。

对应完成列表：[Runtime03 absolute budget completion list](../../../astra/features/runtime/2026-10-13-runtime03-absolute-p95-budget-completion-list.md)。
