# 编译存储规则

本规则用于降低编译体积、核对空间增长来源并纠正浪费。由 Agent 按[直接开发管理标准](agent-artifact-management.md)在任务授权内执行；命令范围和证据要求遵循[里程碑验证政策](../testing-and-validation/milestone-validation-policy.md)。

1. 编译前声明模块、包、features、测试目标/过滤器、profile、命令和理由。普通任务只选最小受影响集合并关闭默认 features；`cargo test`（含 `--no-run`）须选 `--lib` 或一个明确 `--test` 目标。只有任务或所属文档要求时才使用[声明的 workspace/package-set 门](../testing-and-validation/milestone-validation-policy.md#declared-workspace-and-package-set-gates)。不得盲用 `--all`、`--all-features`、`--all-targets`、`--tests`、隐式根目录选择或未声明的逐包循环。
2. 普通 dev/test 使用 `debug=0`、`incremental=false`；普通入口不接受包级或环境变量覆盖来重新启用调试符号和增量缓存。生效的编译环境须进入兼容性身份。显式 release、profiling、shipping、shipping-symbols 保留各自的符号、优化和 panic 契约；特殊诊断另行声明范围、空间预算和保留条件。
3. 构建目录、编译缓存及临时编译输出必须物理位于盘根 `D:\cargo-targets`、`E:\cargo-targets` 或 `F:\cargo-targets` 下，保留 35 GiB 启动余量。项目指定卷时使用指定卷。复用按仓库、工具链、平台、profile、链接/依赖布局和有效配置核验兼容，保留 writer 互斥及 Cargo 原生锁。不要为每个会话、日期或工作树创建完整 target，也不要复制旧产物冒充本次执行。
4. 任务产物清单在产生内容前记录 owner、源码输入、物理路径、任务或原操作身份与保留条件。实际退出码、命令输出与清单分别记录；普通命令回执不证明整棵进程树终止、输入封存或产品产物来源。需要这些证据时按所属产品门单独核验。
5. 获准盘点时比较有效基线，区分最终包、符号、共享构建产物和增量缓存的增长；逻辑大小不等于已分配或可释放大小。清理只处理明确授权、已核对物理身份和保留引用、且无活动 writer 或原生锁的精确对象。保留未知源码、其他会话产物、共享依赖、Git 元数据和历史回执。不要据此创建定时任务、运行自动清理或重放旧请求。

本地入口必须拒绝输出路径逃逸、环境覆盖和超范围选择。命令结果只证明实际执行范围，不替代适用的里程碑、产品与发布验收。
