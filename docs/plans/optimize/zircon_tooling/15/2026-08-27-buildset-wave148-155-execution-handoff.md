# Tooling15 BuildSet 优化执行交接

- 状态：本地实现与测试已完成，Wave148-155 的 361+3 固定运行时异步验证均已提交到独立日志；本会话不轮询协调器或读取这些日志，故不声明集成验收通过。
- 变更范围：BuildSet tracked/validator 热路径采用单流 LFS 检查与 SHA-256、FileInfo 元数据直取、snapshot traversal 的 DirectoryInfo/Attributes 复用、排序 inventory 的 `SequenceEqual` 快路径、Git 参数/行输出的单循环缓冲、NUL split 复用与 capture 生命周期收敛、CLR uppercase hex 转换、index/path/descriptor 精确容量预分配、BuildSetId 三段展开写入及 exact-property 反查消除。
- 本地证据：BuildSet + BuildSummary + ProductInputs 受管 Pester 4.10.1 批次 78/78；BuildSet allocation 合约 19/19；工作流精确计数 361、required script 注册 3、Windows PowerShell 5.1 smoke、PowerShell AST 与 `git diff --check` 均通过。
- 工具微基准（均为本地 synthetic/fixture，不是产品资格数据）：生产端单流 LFS+hash 1,632.1→640.4 ms、69,432,904→38,479,736 B；验证端 776.1→625.6 ms、48,967,632→32,423,088 B；Git 参数构造 1,919.3→415.3 ms、735,844,664→42,483,536 B；stdout 100 行解析 8,790.5→203.0 ms、4,703,284,664→65,523,536 B；CLR hex 转换 5,491.3→1,759.2 ms、1,552,816,072→108,016,040 B。
- 候选处置：split-free index parser、streaming JSON、LFS 短 probe/Ordinal 判定、Task.WaitAll typed/direct、ArrayPool UTF-8 identity 与 HashSet inventory 均因耗时回退拒绝；不纳入本次提交。
- 资格边界：尚未取得 ProductReceipt 绑定的真实产品 P50/P95，因此不把上述工具微基准当作产品性能达标，也不写 promotion/accepted 结论。
