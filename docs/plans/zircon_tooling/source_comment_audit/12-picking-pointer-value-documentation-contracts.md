# 12：拾取指针值的文档契约与生命周期

## 目的

承接[第11阶段](11-input-workspace-documentation-and-lifecycle.md)，按[统一文档规范](04-documentation-contract-and-review.md)审查指针身份、按钮分类与阶段标签。每个声明和语句块均有内容哈希及调用证据；函数、数据结构与变体文档解释职责、使用时机、必要约束、生命周期及适用的 Markdown 参数、返回值和引用。

## 阶段状态

| 项目 | 当前结果 |
| --- | --- |
| 精确源码范围 | `pointer_id.rs`、`pointer_button.rs`、`pointer_phase.rs`，共3文件 |
| 行数与增量 | 28行至215行；18段、187行中文注释，新增空行0 |
| 对象审查 | Root独立记录26条；CORE独立记录26条 |
| 文档覆盖 | 2个手写函数、3个数据结构、10个枚举变体；两个元组载荷在所属声明文档中说明 |
| 程序与原注释 | 删除精确新增段可逐字节恢复审查起点；原始来源不重新标记 |
| 语言与差异 | 3个Rustfmt解析及格式检查通过；精确范围diff检查通过 |
| 正式账本与提交 | 尚未发布、暂存或提交；当前会话未取得账本目录的生效写入授权，索引锁归属未证实 |
| 全仓完成状态 | 未完成；其他模块与正式发布、提交门槛仍待处理 |

## 已确认的关键契约

`PointerId`按值保存调用方选择的完整`u64`，其默认原始值为0。位置、射线、悬停及按钮状态使用这个数值键；相机和视口另由各自记录保存。输入方在需要独立状态时区分身份，并在复用给另一次交互前处理旧状态。构造和读取原始值各有参数、返回值、调用时机、生命周期及引用说明。

`PointerButton`作为指针身份之外的另一维键，按下、释放和拖拽继续携带同一分类。物理来源映射由输入方负责，分类本身不保存按下状态或设备资源；`Other`拥有来源定义的`u16`编码。`PointerPhase`目前仅确认类型定义和公开重导出，文档说明其标签语义及使用者负责的状态边界。

既有`CR-PICKING-0001`的源码标签和问题账本行仍为打开状态。本批只读追踪其按指针合并的证据，未修改标签、创建新编号或宣告修复。

## 核验结果与恢复记录

第一次安装尝试在Rustfmt内存候选检查中失败：相同原始行的插入采用后进先出，导致模块内文档落在类型外文档之后。实际核对三个源码原哈希均未变，失败输出目录不存在。独立r2记录只修正候选顺序，18项文字、作者证据及既有封条均保留；三个修正候选通过解析、格式及逐字节回撤后才安装。

成功安装后，主审完整读取三文件实际diff，确认只含本批新增注释，并将原声明跨度及文档范围绑定到最终哈希。Git报告的LF至CRLF提示已保留在证据中。引用核验限于定义及可见作用域；本批未产生Cargo、Rustdoc、动态测试、Jenkins或里程碑验收结果。

## 证据与来源

逐项审查与错误日志保存在独立封存包中，计划正文仅记录阶段状态。

- [Root独立源码审查封条](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-pointer-values-current-source-review-20261009-r1/seal.json)。
- [CORE独立审查封条](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/w24_pointer_values_core_sol_r11/seal.json)。
- [候选文档复核封条](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-pointer-values-core-and-documentation-addendum-20261009-r1/seal.json)。
- [插入顺序修正及零源码改动回执](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-pointer-values-insertion-order-refinement-20261009-r2/seal.json)。
- [成功安装与语言检查封条](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-pointer-values-declaration-docs-review-and-language-20261009-r2/seal.json)。
- [最终哈希与覆盖绑定封条](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-pointer-values-final-coverage-20261009-r1/seal.json)。
- [正式问题账本](../../../_data/source-comment-audit/issues/framework_navigation_picking.jsonl)与既有标签保持一致。
- [现有Jenkins chat的已准备交接材料](E:/Git/ZirconEngine/.codex/outbox/source-comment-audit-01a0f168-jenkins-boundary-20261009-r1.md)仍未发送；没有投递或验收回执。

## 后续批次

位置与输入、跨帧事件状态、悬停快照及命中分组四个CORE批次已按互不重叠的精确清单并行只读审查，相关测试分别归属所属模块。支持文件在本次注释安装前后的读取哈希独立保留。已封存的四个输入索引文件仍待Root复核和语言检查；其他已交付CORE候选同样不能直接计为最终源码覆盖。

全仓审查仍须逐模块完成最终哈希记录、问题标签一致性、正式账本发布及授权提交。源文件租约、历史来源、外来业务改动和产品排除边界继续按现有证据保留。
