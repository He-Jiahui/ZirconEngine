# 05 — Runtime 当前源码审查与契约注释阶段

## 状态与边界

全仓目标仍在进行，尚未达到完成条件。沿用[证据恢复计划](03-evidence-recovery-and-current-source-review.md)与[统一注释规范](04-documentation-contract-and-review.md)。本计划只记录阶段状态；逐对象记录、精确哈希、调用证据、问题对应关系和安装差量分别保存在结果包与账本中。

当前已分派 18 个互不重叠的只读分区，使用 `gpt-6-luna`、`max`；精确清单合计 253 个文件、17,153 行。每个分区可跨仓追踪调用者，只能在各自获授权的私有结果目录写证据。子审回执须经主审核验当前源码、调用上下文、契约、注释候选和覆盖关系后，才能计入已复核覆盖。

## 已完成本阶段复核的批次

| 批次 | 精确审查文件 | 实际新增注释文件 | 新增注释段／行 | 独立复核记录 | 验证结果 |
| --- | ---: | ---: | ---: | ---: | --- |
| TimeDomains | 7 | 7 | 22／107 | 59 | 7 项 Rust 格式解析、限定 diff、反向字节检查通过 |
| TimePolicy／Fixed | 5 | 4 | 26／192 | 87 | 5 项 Rust 格式解析、限定 diff、反向字节检查通过 |

这两个批次只新增中文注释；保留全部原有字节，反向移除本批精确新增段可还原各自审查起始哈希。没有运行 Cargo 或动态测试，没有新增正式问题编号，没有改变行为或公共 API。以上批次统计不是全仓完成率。

TimeDomains 的[安装结果](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-time-domains-declaration-docs-review-and-language-r1/result.json)与[封条](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-time-domains-declaration-docs-review-and-language-r1/seal.json)分别绑定精确后像和证据。TimePolicy／Fixed 原子审有对象、路径及行号错配，旧包原样保留；当前采用另行建立的[87 项主审记录封条](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-time-policy-fixed-independent-static-review-r1/seal.json)，并另存[安装结果](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-time-policy-fixed-declaration-docs-review-and-language-r1/result.json)及[安装封条](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-time-policy-fixed-declaration-docs-review-and-language-r1/seal.json)。旧错误记录没有改写成新证据。

## 并行审查与结果准入

Clock、AI、Camera、Scene、Text、Input、Navigation、Sound、Window 和 WorldTime 分区正在审查或等待主审。正式准入以逐批核验结果为准，收到 terminal 或候选数量不代表覆盖已通过。

Camera 与 Scene 子审在首次封条已经声明不可改写后又写入同一目录。保留首次回执和最终哈希的差异；当前候选需重新核验，首次封存的可靠性不能由第二次封条恢复。其他包发现的候选事实错误、锚点歧义、支持记录 ID 冲突或额外生成脚本均须单独记录，不能修改已封包。

现有 `TODO:`／`BUG:` 标签、问题编号和调用证据继续保留。后续产品后像按[产品排除清单](../../../_data/source-comment-audit/round-02/ownership/product-exclusions.json)处理，不将业务改动归为注释审查作者。

## 正式账本权限与历史证据

用户已授权增加正式账本目录的写入范围；当前执行环境和配置仍未授予 `docs/_data/source-comment-audit` 写入权，控制配置本身也仍为只读。此前配置写入遭自动审批拒绝，未通过其他工具重试或绕过。已在获授权的私有目录准备[单项授权补丁](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-formal-ledger-permission-proposal-20261006-r1/config-allowlist.patch)、[实际状态回执](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-formal-ledger-permission-proposal-20261006-r1/receipt.json)和[封条](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-formal-ledger-permission-proposal-20261006-r1/seal.json)。该补丁仅为可复核候选；实际配置写入、账本写入与权限操作探针均为零。

权限范围实际生效后，按[文件权限说明](../../../tooling/codex-file-permissions.md)验证对应操作，再将已核验记录写入正式账本。已丢失的历史证据目录不能重新编造；当前独立证据与原有注释来源保持区分。

## 后续工作与完成门槛

继续主审各分区的完整对象与语句块、直接调用、trait 实现、宏生成、动态注册、FFI 入口以及相关测试；用途已经明确的对象可以零新增注释，但必须有对应审查记录。纯注释批次采用对应语言格式或解析、限定 diff、原有字节保留及后像覆盖检查。

按[协调器退役说明](../../../tooling/coordinator-retirement.md)继续独立工作；不恢复旧服务，不调用旧注册、租约、归属、队列或提交 API。阶段提交仍待精确任务差量、共享索引锁诊断及实际验证门槛核验，不能混入既有或其他会话改动。

仅当第一方源码清单的每个文件都有与最终内容匹配的审查记录、待处理文件清零、源码问题标签与问题账本一致，才报告全仓完成。目前这些条件尚未满足。
