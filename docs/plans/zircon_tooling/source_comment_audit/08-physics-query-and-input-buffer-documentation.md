# 08 — 物理查询与输入缓冲声明文档

## 阶段状态

全仓目标保持 **进行中，未完成**。本阶段完成物理查询的 9 个文件和输入缓冲的 2 个文件：新增中文文档、独立核心契约复核、主审复核、Rustfmt 检查及最终内容覆盖绑定。正式账本发布与阶段提交保持待处理。

统一规范继续使用 [04 — 声明文档与审查契约](04-documentation-contract-and-review.md)，前阶段结果保留在 [07 — 音频布局声明文档与逐文件审查恢复](07-audio-layout-declaration-documentation-and-file-unit-recovery.md)。对象、语句块及问题明细保存在独立证据中。

## 本批已核验结果

| 项目 | 物理查询 | 输入缓冲 |
| --- | --- | --- |
| 精确源码范围 | 9 个接口、筛选、模式、查询与命中结果文件 | `frame.rs`、`recorder.rs` |
| 主审对象与语句块记录 | 88 条 | 35 条 |
| 独立核心对象复核 | 88 条 | 22 条 |
| 新增注释 | 63 段、360 行 | 19 段、159 行 |
| 最终行数 | 150 → 510 | 154 → 313 |
| 函数文档附着检查 | 4 个声明 | 9 个声明 |
| 对应语言检查 | 9 个 Rustfmt 检查通过 | 2 个 Rustfmt 检查通过 |
| 原内容与 diff | 精确移除新增片段后逐字节恢复审查起点，限定文件 diff 检查通过 | 精确移除新增片段后逐字节恢复审查起点，限定文件 diff 检查通过 |
| 最终覆盖 | 每个文件及对象范围均绑定最终内容 SHA256 | 每个文件及对象范围均绑定最终内容 SHA256 |

声明文档包含 Markdown 摘要，按实际契约说明职责、使用时机、必要约束、状态与生命周期，以及适用的参数、返回值和可解析参考关系。字段和枚举变体说明单位、身份、筛选或状态关系；私有函数与测试声明适用相同文档深度要求。

物理查询入口为[查询接口](../../../../zircon_runtime/src/core/framework/physics/query_interface.rs)、[筛选契约](../../../../zircon_runtime/src/core/framework/physics/query_filter.rs)和[查询模式](../../../../zircon_runtime/src/core/framework/physics/query_mode.rs)。本批明确查询值与命中快照的所有权、接口与当前实现的边界、首个命中与最近命中的区别、重叠排序的实际依据，以及结果不授予实体存活或物理后端能力保证的约束。`CR-PHYSICS-FRAMEWORK-0001` 和 `CR-PHYSICS-BACKEND-0004` 保留原问题来源及打开状态。

输入缓冲入口为[帧内缓冲](../../../../zircon_runtime/src/input/runtime/event_buffer/frame.rs)和[记录队列](../../../../zircon_runtime/src/input/runtime/event_buffer/recorder.rs)。本批说明相邻移动事件的合并边界、提交时先记录原始事件再进入帧内合并的顺序、帧清理与记录会话的区别、配置改变及零容量时的序号和丢弃统计，以及排空返回值与累计状态的生命周期。

输入记录的[源码字符串检查测试](../../../../zircon_runtime/src/input/runtime/event_buffer/tests/recorder.rs)仍有已证明的静态路径缺陷：其结束分隔符要求 `#[cfg(test)]` 紧接 `mod tests`，当前生产文件在两者间还有 `#[path = "tests/recorder.rs"]`，因此分隔搜索失败并进入 `expect` 的 panic 路径。问题证据已保存，正式编号去重和测试文件审查待完成；本批保持测试及生产行为，新增 TODO／BUG 标签为 0。

## 证据与来源

私有证据根目录为 `E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168`。原子审包、历史注释和业务归属保留；新增作者记录只对应本批精确注释片段。

| 证据 | 私有相对路径 | 封条 SHA256 |
| --- | --- | --- |
| 物理查询注释、格式与原内容核验 | `results/root-physics-query-declaration-docs-review-and-language-20261008-r1` | `75792190dda49ef2ea3fd4138f194531393023071c6d88cdfbdba383afea9bea` |
| 物理查询最终覆盖 | `results/root-physics-query-final-coverage-20261008-r1` | `0536a0930f521bafbb1b2aa1234269a6628db5529bdadaa92570d92f56ecc7f3` |
| 输入缓冲注释、格式与原内容核验 | `results/root-input-buffer-declaration-docs-review-and-language-20261008-r1` | `ab28e98d8d2fb1322864184c0e463d85e32fd696cb2439c629c1c1313118fd17` |
| 输入缓冲最终覆盖 | `results/root-input-buffer-final-coverage-20261009-r1` | `504b5468a866383e437b9ce9c42847b31b5e2a70a6485c2b19b9c09ef4531a42` |
| r7 精确任务准备 | `jobs-wave24-terminal-unit-recovery-r7` | `cf66e115cdef9a9914586e2ebbdb009d0b2d8a575f405b714dd62a4e2f41a792` |
| r7 原生指派回执 | `results/root-wave24-r7-native-dispatch-receipt-20261009-r1` | `f112c4f51b79196f08e8e33943123d5240f92a0f151caae6665971fa67ea06f3` |

## 并行审查恢复

恢复边界的原生列表仅有主审。已封存的 r4、r5 文件单元保留，未完成或未封存项保持待处理，旧草稿不授予完成状态。r7 已实际指派 11 个 `gpt-6-luna max` 审查任务和 3 个 `gpt-6.1-sol max` 核心契约复核任务，共 24 个互不重叠的精确文件、1,223 行起始源码。

新任务只读取源码并写各自获授权的私有结果目录；跨模块读取仅作为调用证据。每个完整文件单元及时封存，主审在独立复核、对应语言检查和最终覆盖核验后安装注释。任务消息、工具视图、runner 和模型容量的实际失败另记在指派回执中；成功指派不等于完成审查或验证。

## 验证与剩余条件

本批验证限于静态契约、Rustfmt、精确内容还原、限定 diff 和结构检查；Cargo、动态测试及 Rustdoc 构建未运行。全工作区、里程碑、性能和生产迁移接受条件保持各自状态。

当前分支 `main`，缓存区为空；`.git/index.lock` 为零字节，所有者尚未证明，锁及共享索引完整保留。阶段提交和附带通知仍待处理。

用户已授权扩大正式账本写入范围，但当前配置与会话实际写入条目仍未匹配 `docs/_data/source-comment-audit`。最终覆盖和发布准备保存在获授权私有目录，正式发布保持待处理。遵循[文件权限规则](../../../tooling/codex-file-permissions.md)及[协调器退役与恢复规则](../../../tooling/coordinator-retirement.md)，继续保留历史归属、回执、外来业务修改与实际验收边界。

全仓完成仍要求全部第一方源码具有匹配最终内容的审查记录、待处理文件清零、源码 TODO／BUG 与规范问题账本一致，以及必要复核、验证和阶段提交完成。
