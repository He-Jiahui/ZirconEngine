# 06 — 声明文档补充与 Runtime 实现审查

## 阶段状态

全仓审查继续进行，尚未达到完成条件。沿用[证据恢复计划](03-evidence-recovery-and-current-source-review.md)与[声明文档规范](04-documentation-contract-and-review.md)。函数、类型和接口的说明包含适用的 Markdown 摘要、参数、返回值、调用时机、必要约束、生命周期与参考关系；手写 trait 方法也逐项核对。阶段计划只保存状态，逐对象证据与精确增量保存在独立结果包中。

## 相机协议声明补充

已独立复核 `core/framework/camera_controller` 的 15 个文件，包括构造方法、10 个手写 `Default` 方法、55 个公开字段、枚举分支及相关语句块。主审分别读取 Runtime 实现、Editor 与 Dynamic 调用上下文、测试、源码守卫和当前模块契约，再确定初始化、跨帧状态、坐标与单位、输出快照以及光标意图的边界。

| 批次 | 精确文件 | 新增文档段／行 | 主审记录 | 实际验证 |
| --- | ---: | ---: | ---: | --- |
| 相机协议声明 | 15 | 30／389 | 154 | 15 项 Rustfmt、限定 diff、反向字节检查通过 |
| 手写 `Default` 声明补充 | 上述文件中的 10 个 | 10／109 | 10 | 10 项 Rustfmt、限定 diff、反向字节检查通过 |

两批合计 40 段、498 行新增注释；最终覆盖仍是 15 个文件。最终哈希复核与组合反向恢复均通过，全部起始源码和原有注释字节保留。36 个类型／函数声明的文档邻接检查是结构证据，文档含义由逐项主审记录支撑。引用按已读定义与作用域静态核验，未运行 rustdoc 构建、Cargo 或动态测试。

两批各自的[声明安装回执](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-camera-declaration-docs-review-and-language-20261007-r1/result.json)和[`Default` 安装回执](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-camera-default-method-docs-review-and-language-20261007-r1/result.json)保持独立。[最终覆盖链](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-camera-final-coverage-chain-20261007-r1/coverage-final.jsonl)及其[封条](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-camera-final-coverage-chain-20261007-r1/seal.json)把不可改写的审查起点、新增片段和当前后像关联起来。

早先 Camera 子审封条改写的事实继续保留；首次封条没有被第二次封条恢复为有效证据。本阶段使用另存的[154 项主审记录](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-camera-independent-static-review-20261007-r1/seal.json)与[10 项声明补充记录](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-camera-default-method-static-review-20261007-r1/seal.json)，没有改写原子审包或原有作者信息。既有 `CR-FRAMEWORK-CAMERA-0001` 问题及其 `TODO:` 保留；没有新增正式问题标签或业务修复。

## 并行实现审查

已分派 20 个互不重叠的只读分区，使用 `gpt-6-luna`、`max`，合计 182 个文件、15,632 行。范围包括 Input 动作评估与索引、管理器、状态、事件录制与路由、相机实现、Net、Physics、Platform 以及 Picking 的实现和关联测试。

分区清单与[统一工作约定](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/wave24-readonly-review-instructions-root-20261007-r1.md)保存在获授权的私有目录。各子审只能写自己的证据目录，可跨仓只读追踪调用者；源文件由主审核对当前哈希、说明事实、插入位置、问题证据和语言检查后再安装。相机实现子审保留实际读到的早先协议哈希，协议文档新后像另存链证据。

这些分派数量不是已通过覆盖数。上一阶段与当前阶段的候选仍须逐批准入；收到终态、候选数量或邻接扫描通过均不能代替独立内容复核。

## 账本、共享索引与提交

正式账本目录的新增写入范围已获用户授权，当前配置和执行环境仍未授予该目录写入权。继续沿用[单项授权候选与状态回执](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-formal-ledger-permission-proposal-20261006-r1/seal.json)；实际配置写入、正式账本写入和权限操作探针均为零。已核验的覆盖记录暂存私有目录，正式发布仍待范围生效及对应操作验证。

本阶段只读核验当前分支为 `main`；暂存差异为空，共享索引锁存在且为零字节。锁的归属尚未确认，保持原样；阶段提交待精确任务暂存差量、索引归属与实际验证门槛核验。没有暂存、提交、推送或发送企业微信通知。

按[协调器退役说明](../../../tooling/coordinator-retirement.md)继续独立工作，不调用旧注册、租约、归属、队列或提交接口。保留[产品排除与自然边界记录](../../../_data/source-comment-audit/round-02/ownership/product-exclusions.json)，后续业务后像不归为注释审查作者。

## 后续与完成条件

继续逐批复核已收候选和并行实现审查，按文件最终内容维护覆盖与问题关系。每个第一方源码文件均须有匹配最终哈希的审查记录；说明不足的声明继续补充，确实无需新增的对象保留有证据的决定。

只有全部第一方源码审查记录、最终覆盖、语言检查、问题标签一致性与待处理清零条件满足后，才报告全仓扫描完成。当前目标保持进行中。
