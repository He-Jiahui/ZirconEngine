# Jenkins 编译存储规则

这是用户 2026-10-18 要求降低编译体积、追踪浪费来源并强制纠正的执行约束。
Jenkins 管理版本、构建、复用和清理；Codex 的原每日任务负责差异审查和纠正交接。

1. 编译前声明本次实际模块、包、features、测试目标/过滤器、profile、命令和原因。
   只选所需显式包与 features，并关闭默认 features。禁止 `--workspace`、`--all`、
   `--all-features`、`--all-targets`、`--tests`、隐式根目录全量构建和逐包循环全覆盖。
   `cargo test`（含 `--no-run`）须选 `--lib` 或一个明确 `--test` 目标。
2. 普通 dev/test 使用 `debug=0`、`incremental=false`，包级覆盖不能重新打开。
   Jenkins 管理执行环境在选择兼容池和运行前统一设置 `CARGO_INCREMENTAL=0`、
   `CARGO_PROFILE_DEV/TEST_INCREMENTAL=false` 和 `CARGO_PROFILE_DEV/TEST_DEBUG=0`。
   这些实际环境参数必须参与池身份，不能与旧的大体积设置共用兼容性身份。
   release/profiling/shipping 的符号、优化和 panic 语义按其原产品契约保留。
   普通请求不接受以环境变量私自重新开启增量缓存或调试符号；需要诊断时另行
   提交明确的诊断需求、空间预算和保留条件，不能静默放宽普通构建入口。
3. 只复用 Jenkins 按工具链、平台、profile、features、依赖及有效环境选择的共享池。
   不按 session、日期或轮次创建完整 target；不复制旧产物来冒充复用。产物和
   编译器缓存只能物理位于盘根 `D/E/F:/cargo-targets`，项目指定 E 根时使用 E。
   35 GiB 构建余量、真实 writer/读引用及终结证据继续有效。
4. `artifacts.json` 在产生内容前登记 owner、原 request/operation、源码身份、
   物理输出与保留条件。实际命令、原生终结和清理回执用于确认执行，不以请求
   提交或清单文字代替。无归属、失败或未知状态保持待处理。
5. 每日按[原 Codex 巡检协议](jenkins-transition-monitor.md)比较有效基线并定位来源。
   增长是正常编译、符号、增量缓存还是重复兼容池，需要分别举证；逻辑大小不
   等于实际分配或可释放大小。确认违规后由 Jenkins 执行范围修正和有界维护，
   保留未知源码、共享依赖、其他会话活动资源、Git 元数据和历史回执。

公开 Cargo 解析器与管理执行器均拒绝超范围 recipe，直接提交 JSON 不能绕过。
历史请求仍可按原身份查询；新规则不抹掉失败、不自动重放 UNKNOWN 请求，也不
把普通 Git 提交改成强制全量编译门禁。部署是否生效看当前胶囊和真实入口回执，
单有本文或测试 fixture 不算已完成运行接管。
