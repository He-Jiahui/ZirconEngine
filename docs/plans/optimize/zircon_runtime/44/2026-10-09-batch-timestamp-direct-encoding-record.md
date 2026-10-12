# Runtime44：批次时间戳直接编码优化记录

日期：2026-10-09。状态：四文件源码优化已采用；原生功能、产品及完整性能待验证。

## 已采用的优化

每个非空日志批次在原预留后、FIFO drain 前采样一次 Local 时间；以原数字/字面格式直接写入现有 Vec 的 fmt::Write 适配器。按实际位置差取得首记录前缀范围，后续记录从同一 Vec 复制该范围，不假定年份宽度固定为 19 字节。空批次保持不采样、不输出。

原目录 current_log_timestamp 函数体保持。level、scope、message escaping，FIFO，write/flush 错误计数、delivery metrics，clear 和大记录储存释放保持原逻辑。直接 formatter 消除批次时间戳的临时 String 构造；Local 时钟自身分配及实际 allocation / CPU 收益尚未测量。

新增两个有意义的日历回归用例，与旧 DateTime formatter 比较完整输出字节，覆盖午夜、闰日/闰秒、epoch、负数与扩展年份、非 UTC offset 及已有缓冲区前缀和同 Vec 复制。原测试名称、断言和 ignore 状态保持。

## 源码验证

[独立核心评审](../../../../../.codex/outbox/runtime44-batch-timestamp-direct-encoding-core-review-v1/review.json)和[实际采用回执](../../../../../.codex/outbox/runtime44-batch-timestamp-direct-encoding-v1/actual-canonical-apply.json)。

47 个输入两次核对、43 个非修改依赖保持；实际四文件 Rust 2021 格式及 scoped diff 检查通过。共享索引、旧索引锁内容及元数据、Cargo 和控制配置保持。Root 首次将 binary index 按文本归一化的安全拒绝保留，当时没有源码写入；实际采用使用 binary exact-raw 校验。

实际采用回执 SHA-256：e8885987d19ec92deb4c42911976859d8e8b359c7093d22fe5c3249e0af8699e。

## 批量验证与待完成标准

沿用原 15 arms；只在 Runtime #2 加两个精确名称，Diagnostic 名单 16→18，该 arm 的保守最低匹配 18→20。#9 原 producer 和 #15 最低 146 保持；最终组合批次另写不可变草案，不声明 V8 已改变。

原完整 54 行负载矩阵、20 个关键提交样本、2ms admission timeout、p95 ≤50ms、1000–1250ms 负载时长、RSS ≤128MiB，以及原 conservation、backpressure、drop、queue4096 / batch256 / flush25 和非零 Windows RSS 采样要求全部保留。实际 allocation、CPU、真实 NVMe/console I/O 和产品生命周期资格均待验证；不能以源码及 synthetic 结果代替原验收。

对应[计划完成列表](../../../astra/features/runtime/2026-10-09-runtime44-timestamp-completion-list.md)。
