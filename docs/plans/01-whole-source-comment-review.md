# 01 — ZirconEngine 全仓源码注释审查

任务 ID：`source-comment-audit-20261012`；主审聊天：`01a13045-75e3-7dd2-86d8-de54d914c232`。目标是不改行为和公共 API，逐项核对全部第一方源码的调用关系、设计意图并补充必要中文注释。

规范、覆盖账本、问题账本及逐项证据保存于 `docs/tests/source-comment-review-*`；本计划只记录阶段状态和验收引用。

| 阶段 | 范围 | 状态 | 验收证据 |
| --- | --- | --- | --- |
| 0 | 完整清单、哈希账本、规范、所有权与 Jenkins 交接 | 进行中 | [规范](../tests/source-comment-review-standard.md)；原请求 `source-comment-audit-20261012` |
| 1 | 公共契约、核心、反射与 Runtime host 边界 | 会话/预算 9 文件已独立复核；68 文件归属待核对，29 文件审查中 | [覆盖账本](../tests/source-comment-review-coverage.json) 已登记 106 个文件；[会话复核](../tests/source-comment-review-independent-core-session-003.json)；[预算复核](../tests/source-comment-review-independent-reflect-budget-002.json)；三名 Luna max 与一名 Sol max 并行 |
| 2 | Runtime 子系统与相关测试/着色器 | 待开始 | — |
| 3 | Editor 与相关测试/资源 | 待开始 | — |
| 4 | 第一方插件与相关测试/资源 | 待开始 | — |
| 5 | Hub、入口、工具、示例、执行模板与其余源码 | 待开始 | — |
| 6 | 最终清单/hash、标签、提交、产物收尾对账 | 待开始 | — |

使用共享 `main` checkout，初始 HEAD 为 `21da363625e644aacca45b8c90621547cd42ba5b`。每批重新核对当前 HEAD、已有 diff 和内容 hash，不以初始状态推定归属。既有与外来改动均保留。

索引锁的分时读取曾观察到零字节锁，也有未发现锁的结果；不能据此推定锁失效或完成诊断。首次提交由 Jenkins 重新核对实际锁、索引所有权及活进程身份，不删除或绕过锁。

原先核验的正式入口仅含 `validate`。最新 Jenkins 观测显示入口及接纳源码的物理哈希与安装 pin 不匹配，原运行时身份尚待核对；当前交接未被正式接纳。完整清单生成、批次格式/解析票据和精确归属提交仍待正式能力及实际回执；独立源码审查继续。

用户授权协调器阶段提交和附带企业微信自动通知；Jenkins 目标聊天 `01a0f6e1-e9a8-7b92-aed8-5220c71a9c43`，按指定 `.jenkins/records` 交接。原请求已由 `zircon-records/25` 接纳并核验归档身份；完整清单补充由 `zircon-records-followup/3` 接纳，首个反射批次由 `zircon-records-followup/4` 接纳，外部源码变化与批次范围对账由 `zircon-records-followup/5` 接纳。控制记录为 `.jenkins/state/records-queue/controls/source-comment-audit-20261012.25.json`，最近核验仍为 `waiting`、`done=false`、`formalAcceptance=false`，没有验证或提交回执。接纳与归档不能替代源码审查验收。

已分配清单中的 68 个先前范围在交接期间回到当前 HEAD 的源码文本，旧复核与验证仅对应历史哈希，相关批次保持 stale 或等待外来归属。[Jenkins 观测响应](../../.jenkins/records/source-comment-audit-20261012-assigned-source-drift-20261013.response.json) 未确认重写者或原操作终态；这项结论限于已观测路径。帧接口、反射预算、会话接口及 Host 请求接口的 26 个独立文件继续审查。全局账本持续更新，提交前必须逐路径重新核对。

会话与预算两个批次的最终物理哈希、对象/调用证据及纯注释差异已由主审独立核对；9 文件仅新增 485 行注释或空行，原始字节行和行尾保持。已有格式及结构证据只用于相同最终哈希，正式 Jenkins 票据、精确提交及通知仍待处理。操作/插件事件镜像及完整相关测试新增 5 个精确归属文件；其他会话的 dirty 文件与旧 68 个范围保持只读。新的普通交接附带 reflog 的 `reset: moving to HEAD` 观测，只用于调查归属，不能证明重置方式、执行者或旧进程终态。

IME composition v2 的协商、二进制事件、App 配置与相关测试新增 7 个独立文件；每个公开归属文件遵守现有总行数 700 的守卫，所有对象都审查，已清楚的对象可不新增说明。事件文件原有 653 行，新增说明优先覆盖模块/公共边界，不删旧行或变更结构以放宽限制。
