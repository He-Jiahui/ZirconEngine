# 15：拾取策略、指针输入与位置、射线测试的契约文档

## 目的

承接[第14阶段](14-picking-hit-record-and-target-documentation-contracts.md)，按[统一文档规范](04-documentation-contract-and-review.md)审查拾取策略、指针位置与动作、共享夹具和射线测试。全部手写函数、数据类型、字段和枚举分支的中文文档说明职责、使用时机、参数、返回、必要约束、生命周期和参考契约；逐项审查、实际失败及恢复记录保存在独立证据包。

## 阶段状态

| 项目 | 当前结果 |
| --- | --- |
| 精确源码范围 | `core/framework/picking/{pickable,pointer_location,pointer_input}.rs`、`tests/picking/mod.rs`和`tests/picking/rays.rs`，共5文件 |
| 行数与增量 | 303行至811行；51段、508行中文注释，其中506行文档注释、2行私有说明；新增空行0 |
| 对象审查 | Root独立记录114条；CORE独立记录114条；全部绑定最终源码哈希和位置 |
| 声明文档 | 14个手写函数、5个数据类型、12个字段（含枚举载荷）、7个枚举分支、3个常量及5个模块 |
| 候选复核 | 24段采用；25段根据实际调用证据修订；Root另补2段增量字段文档；129项引用绑定核对定义与作用域 |
| 程序与原注释 | 在内存中精确删除新增片段，恢复全部起点字节；原程序、原注释及其来源保留 |
| 语言与差异 | 5个Rustfmt解析及精确格式检查通过；范围内diff检查通过，主审完整读取实际diff |
| 正式账本与提交 | 尚未发布、暂存或提交；当前聊天权限未包含正式账本目录，索引锁归属未证实 |
| 全仓完成状态 | 未完成；其他分区、正式发布及提交门槛继续开放 |

## 已确认的关键契约

拾取策略的可悬停与阻挡标志相互独立。策略值可复制，不持有场景资源；构造和公开字段不执行命中排序或资源查询。实际悬停投影先决定目标是否进入结果，再根据阻挡标志决定是否终止；手工建立快照的入口沿用调用方提供的值。

指针位置拥有身份、视口句柄和局部像素坐标。构造器不验证资源存续、坐标配对或有限性。范围判断采用包含边界的闭区间，尺寸转为浮点值后可能舍入；零尺寸下原点仍可通过范围判断，射线构建另有准入检查。

指针动作只保存输入源提供的载荷，按序消费与跨帧状态由派发器拥有。零移动增量跳过移动处理，移动事件中的增量与由位置差计算的拖拽增量具有各自来源。滚动标签及增量原样传递，单位转换由生产方和消费方约定。取消先影响整帧悬停筛选；全部输入位置先构建索引，同指针后续输入的位置仍可影响该索引。

共享夹具创建自有相机、命中和位置值，固定编号不注册或保活设备、视口和相机。标签投影返回新向量，保持输入事件的顺序和重复项，不保留输入借用。

五个射线测试明确固定透视配置、数值容差、相机源活动标志、视口配对和射线键的断言范围。多视口射线键测试没有收集后端命中或推进事件；下游命中合并与事件路由继续由相应模块审查。

## 问题与证据边界

本批没有新增正式`BUG:`或`TODO:`标签。`CR-PICKING-0001`只保留既有问题引用，原归属和未关闭状态不变。其他批次的只读观察与待验证项继续保留各自状态。

写入前格式检查发现原移动变体会被收成一行，已通过补充有实际解释价值的增量字段文档保留原排版。最终覆盖统计按实际公共、私有夹具和测试声明类别核验；首次失败及未执行的适配器原字节保存在证据中。全部反向恢复均在内存中完成，未建立完整源码快照或重建已丢失的历史目录。

引用核对只证明当前定义和作用域。Rustfmt结果不包含Rustdoc构建、类型检查、宏展开、链接、动态行为或性能验收。

## 证据与来源

- 拾取策略：[Root审查封条](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-pickable-current-source-and-contract-review-20261009-r2/seal.json)、[CORE审查封条](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/w25_picking_pickable_core_sol_r1/seal.json)、[语言检查封条](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-pickable-declaration-docs-review-and-language-20261009-r2/seal.json)、[最终覆盖封条](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-pickable-final-coverage-20261009-r1/seal.json)。
- 指针输入、位置及测试：[Root审查封条](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-pointer-input-location-current-source-and-contract-review-20261009-r3/seal.json)、[CORE审查封条](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/w25_picking_input_location_core_sol_r1/seal.json)、[字段文档补充封条](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-pointer-input-location-field-documentation-addendum-20261009-r1/seal.json)、[语言检查封条](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-pointer-input-location-declaration-docs-review-and-language-20261009-r4/seal.json)、[最终覆盖封条](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-pointer-input-location-final-coverage-20261009-r4/seal.json)。
- [现有产品排除清单](../../../_data/source-comment-audit/round-02/ownership/product-exclusions.json)保持只读；当前批次不包含所列路径。CORE起点、Root新增注释片段和既有业务字节的来源分别保留。

## 后续批次

射线映射、报告、管线、基础几何后端、事件状态、悬停与命中投影及输入动作等CORE包继续等待Root独立复核和语言检查。输入值、事件保留、游戏手柄设备与震动三个互不重叠的分区正在并行只读审查，相关调用与测试按所属模块追踪。

本阶段没有Cargo、Rustdoc、动态测试、Jenkins或里程碑验收结果，没有外发、暂存或提交。正式账本目录的既有用户授权仍待当前权限生效，现有索引锁保留；现有Jenkins聊天的交付仍未发送。全仓完成仍要求最终哈希覆盖、源码问题标签与账本一致、待处理文件清零、正式发布及授权提交；旧协调器保持退休。
