# 13：默认动作管理器的服务、锁与返回值契约

## 目的

承接[第12阶段](12-picking-pointer-value-documentation-contracts.md)，按[统一文档规范](04-documentation-contract-and-review.md)复核默认输入动作管理器及实际挂载的中毒恢复测试。文档覆盖职责、调用时机、参数关系、返回值、必要约束和生命周期；逐对象记录与恢复日志保存在独立证据包。

## 阶段状态

| 项目 | 当前结果 |
| --- | --- |
| 精确源码范围 | `input/runtime/default_input_action_manager.rs`及`input/runtime/tests/default_input_action_manager.rs`，共2文件 |
| 行数与增量 | 132行至402行；18段、270行中文文档，新增空行0 |
| 对象审查 | Root独立记录52条；CORE独立记录51条；主审另列测试匿名闭包 |
| 声明文档 | 11个手写函数、1个结构、1个字段及两个模块；无参测试使用“测试场景”章节 |
| 程序与原注释 | 精确删除新增片段恢复全部起点字节；不重新认领原注释和外来业务字节 |
| 语言与差异 | 2个Rustfmt解析及格式检查通过；范围内diff检查通过，主审完整读取实际diff |
| 正式账本与提交 | 尚未发布、暂存或提交；当前权限未包含正式账本目录，索引锁归属未证实 |
| 全仓完成状态 | 未完成；其他批次、正式发布及提交门槛仍待处理 |

## 已确认的关键契约

模块工厂按有效配置构造管理器，另用共享服务包装完成注册。直接构造或派生默认实例只建立求值器及外层互斥锁。映射读取、替换和六种求值入口在同一外层锁上串行化；已持锁的独占求值器借用通过`get_mut`使用内部工作区。中毒恢复取回guard，不清除中毒标志，也不验证或修复状态不变量。

映射读取返回完整拥有的副本，修改后须显式安装。映射替换先编译新代际，再替换映射、代际并重置工作区逻辑内容，保留可复用容量。求值的帧、活动上下文和消费输入只借用至本次调用结束；返回动作状态拥有各自的名称、集合和值表，后续求值或重绑定不会改写已返回值。

空活动列表表示使用全部启用上下文；配置禁用位继续有效。没有命名上下文的动作不受活动名称列表限制。已消费按钮会跳过含该按钮的整条绑定，已消费轴逐来源过滤。恢复测试声明了没有任何绑定的动作，其断言不覆盖配置中存在空按钮、空轴绑定的缺陷。

## 问题与证据边界

既有`CR-INPUT-0001`与`CR-INPUT-0003`源码标签和问题账本仍开放：前者是外部直接装载空绑定导致持续按住，后者是服务测试缺少平台依赖、在激活前失败。本批未修复行为、改变标签或宣告测试通过。

主审另外确认一条文本守卫不匹配：守卫读取生产管理器，却要求仅存在于挂载测试中的测试函数名。该观察保存在私有问题记录中，等待所属守卫批次和正式发布；生产文档未补造测试名称。另一守卫要求专用求值调用名称恰好出现六次，插入前后计数一致。

CORE初读工作区的哈希与其后Root文档后像分别保留。主审仅在内存中删除已封存的28段Root工作区文档，精确恢复初读哈希和支持跨度；没有写入旧源码、生成完整快照或重建已丢失的历史目录。路径读取失败、候选字段误读及输出截断均保留，并有实际恢复证据。

## 证据与来源

- [Root独立源码审查封条](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-default-manager-current-source-review-20261009-r1/seal.json)。
- [CORE独立审查封条](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/w24_default_action_manager_core_sol_r9/seal.json)。
- [候选、引用与工作区来源复核封条](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-default-manager-core-and-documentation-addendum-20261009-r1/seal.json)。
- [注释安装及语言检查封条](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-default-manager-declaration-docs-review-and-language-20261009-r1/seal.json)。
- [最终哈希及逐对象覆盖绑定封条](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-default-manager-final-coverage-20261009-r1/seal.json)。
- [正式问题账本](../../../_data/source-comment-audit/issues/input_engine.jsonl)保持只读，现有问题未关闭。
- [现有Jenkins chat的已准备交接材料](E:/Git/ZirconEngine/.codex/outbox/source-comment-audit-01a0f168-jenkins-boundary-20261009-r1.md)仍未发送，没有投递或验收回执。

## 后续批次

位置与输入、事件状态、悬停快照及命中分组四个CORE包已封存，均为只读候选，Root和语言验收仍待完成。射线与射线映射、拾取管线及基础几何后端按新的精确清单继续并行只读审查，相关测试随模块归属。输入索引等其他已交付包继续等待独立主审。

本阶段没有Cargo、Rustdoc、动态测试、Jenkins或里程碑验收结果。全仓完成仍要求最终哈希覆盖、问题标签一致性、待处理文件清零、正式账本发布和授权提交；既有产品排除与来源边界继续保留。
