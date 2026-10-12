# 10：输入求值代际与事件录制的声明文档

## 范围与规范

本阶段承接 [09 阶段](09-picking-contract-documentation-and-bounded-parallel-review.md)，按 [04 文档规范](04-documentation-contract-and-review.md) 复核输入求值代际及其测试、事件缓冲入口及录制测试。函数和数据结构的文档说明职责、使用时机、必要约束和生命周期，并在适用时提供参数、返回值及参考章节；首段作为摘要。私有声明、trait 实现和测试辅助函数同样审查。

源码仅添加中文注释。现有程序和注释字节以精确移除本阶段插入内容的方式核验；既有业务内容的作者归属保持原记录。逐对象、语句块、调用证据及问题记录放在独立审查产物中，本计划只记录阶段状态。

## 已完成的模块批次

| 批次 | 文件数 | 起始／最终行数 | 新增注释段／行 | 函数文档 | 主审／核心审查记录 |
| --- | ---: | ---: | ---: | ---: | ---: |
| 输入求值代际与测试 | 2 | 356／694 | 35／338 | 17 | 75／85 |
| 事件缓冲入口与录制测试 | 2 | 142／312 | 15／170 | 7 | 40／40 |
| 本阶段合计 | 4 | 498／1006 | 50／508 | 24 | 115／125 |

以上是本阶段精确文件集合的数量。主审与核心审查对语句块的拆分粒度不同，两个记录数分别保留。它们不表示全仓覆盖率。

四份文件均通过对应 Rust 语法／格式检查、限定文件的 diff 空白检查、插入内容精确反向还原检查及最终内容哈希绑定。未运行 Cargo、动态测试或 Rustdoc 构建；文档引用已按定义及作用域静态复核。

### 输入求值代际

文档明确动作映射、编译索引、上下文槽和工作区的配对关系；说明临时借用的名字在构建结束时释放，返回的代际持有自身的数据。固定输入比较、源码文本检查和计时样例分别说明其输入范围和验证边界。

- [主审复核与纠错证据](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-action-generation-current-source-review-20261009-r2/seal.json)，SHA256：`c6cd438643f3ecf4bef951f38021504800fcee0e2da8779c9efb05616deade46`。
- [核心复核](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/w24_action_generation_core_sol_r8/seal.json)，SHA256：`b1b48a9bcb5a7bf0c3897a0e33e68cd9849acf184c17e289039bd1c35b63a2f5`。
- [注释写入与语言检查](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-action-generation-declaration-docs-review-and-language-20261009-r1/seal.json)，SHA256：`98a0875490bb252ab92190a703d1756537b0dd67a9a34a10d8132ca85df9bcbf`。
- [最终覆盖绑定](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-action-generation-final-coverage-20261009-r1/seal.json)，SHA256：`d4f7ec65e82b455a72d951d703fc39a8a84982cb974f2841309f1501be49e90a`。

### 事件缓冲入口与录制测试

文档区分帧缓冲与原始录制周期，说明排空、关闭、持续启用和缩容对状态的影响。测试文档覆盖队列与计数的归属、容量裁剪、返回值、panic 路径，以及计时区间内外的成本。

- [主审复核](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-event-recording-current-source-review-20261009-r1/seal.json)，SHA256：`946dac7be0a41d37acc9c9a0b81b24e084243fd54276da9d2a11d55594cea51d`。
- [核心复核](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/w24_event_recording_core_sol_r8/seal.json)，SHA256：`5d6a51051cd86097f4be1d4290956f916541f195b7459b1f78498ae0c037bdb0`。
- [注释写入与语言检查](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-event-recording-declaration-docs-review-and-language-20261009-r1/seal.json)，SHA256：`672dd014e1ffcd9a88802a77b1d316e61609df7a56f9ecf7d3fc668a5bb2eb1a`。
- [最终覆盖绑定](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-event-recording-final-coverage-20261009-r1/seal.json)，SHA256：`0e036848c24879bda3ea0532846de28c36b9ca313b2e666c6dbbb4d6090ef1ae`。

## 问题与审查边界

既有 `CR-INPUT-0001` 保持开放，原标签及原作者记录保持原状。录制测试的既有私有观察 `F007` 也保持开放：父源码的 `#[path]` 位于 `cfg(test)` 与测试模块声明之间，第一处精确文本分隔缺失，后面的 `expect` 会先于结构断言触发失败。静态可证明的路径与动态测试结果分开记录；本阶段没有安装新问题标签，也没有修复业务行为。

业务排除清单沿用原记录和哈希。Editor06、Runtime11、Sound、SDK 及工具测试的历史自然边界和产品内容不因本阶段改变作者、覆盖证据或提交归属。失去的旧临时证据没有重建成历史快照。

## 并行审查状态

工作区及其测试的两份文件已交给 `gpt-6.1-sol max` 独立核心复核。原文件单元保留原封条；新记录修正声明锚点和属性前的文档位置，并核验新的调用方内容哈希。生产单元遗漏脚本封印的证据缺口单独记录；未封印的修正 JSONL 和无效 JSON 附录不作为已通过的审查产物。

另已按互不重叠的清单派发三个 `gpt-6-luna max` 只读任务：指针值类型、命中记录与目标、指针语义事件。三次新任务均报告消息载荷不可读，未交付审查记录或封条；这些分区仍为待处理。Root 与核心复核继续推进独立工作。

动作管理器的错误路径比较已纠正：原清单指向 `input/runtime/default_input_action_manager.rs` 及其测试，之前误用了 `core/framework/input/input_action_manager.rs` 接口文件。因此那次哈希检查失败不能证明目标源码变化或业务作者变更；未改动任何管理器源码。该分区继续按原精确文件清单审查。

## 待完成事项与完成条件

正式账本目录的写入授权仍未出现在本聊天的生效权限中；本阶段记录保存在已获授权的私有目录，尚未发布到正式账本。现有 Git 索引锁的归属尚未确认，阶段提交继续保留为待处理。本阶段没有暂存、提交、推送或发送外部通知，也没有调用已退役协调器。

继续复核工作区、动作管理器、RPC 契约和指针分区，逐批完成语言检查和最终内容绑定。待全部第一方源码文件有匹配最终哈希的审查记录、问题标签与问题账本一致、待处理文件清零，并完成适用的提交与验收后，才能报告全仓扫描完成。
