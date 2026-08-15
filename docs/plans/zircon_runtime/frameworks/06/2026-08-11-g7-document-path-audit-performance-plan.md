---
related_code:
  - tools/check_conventions.py
  - tools/convention_exemptions.py
  - tools/tests/test_check_conventions.py
  - tools/tests/check_conventions/document_paths.py
implementation_files:
  - tools/check_conventions.py
  - tools/convention_exemptions.py
plan_sources:
  - docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
  - docs/plans/engine-code-structure-convention.md
tests:
  - tools/tests/test_check_conventions.py
  - tools/tests/check_conventions/document_paths.py
doc_type: performance-plan
status: implementation_complete_secondary_review_green_performance_attestation_pending
---

# G7 文档路径审计性能计划

## 目标与边界

本计划只优化 Frameworks06 G7 文档路径审计，不改变 `related_code`、
`implementation_files`、`tests` 的判定合同，不把远程 URI、命令、glob、模板或构建产物
重新解释为仓库 owner，也不弱化绝对路径、仓库逃逸、符号链接或 junction 逃逸检查。

这是开发基础设施优化，不是运行时引擎性能证据。本计划不声明帧时间、吞吐、RSS 或功耗改善，
也不以一次墙钟结果替代稳定测量。

## 当前实现与测量

Windows Python 3.14 `cProfile` 对当前共享 `main` 的一次完整扫描得到：

| 指标 | 当前值 |
|---|---:|
| 文档数 | 2,314 |
| 声明路径数 | 73,858 |
| 唯一路径数 | 16,502 |
| 唯一父目录数 | 3,567 |
| 总墙钟 / profile 时间 | 38.50 s / 38.50 s |
| `_path_violation_reason` 累计时间 | 29.69 s |
| `Path.resolve` 累计时间 | 21.13 s |
| Windows `_getfinalpathname` self time | 18.77 s |
| front matter 读取与解析累计时间 | 4.73 s |
| `Path.exists` 累计时间 | 0.81 s |

profile 共记录 8,999,086 次调用。现有完整路径缓存已把 73,858 条声明压缩为 16,502 次
文件系统判定，但每个唯一路径仍调用一次 full `Path.resolve()`；Windows 最终路径查询占总时间
约 48.8%，是当前最低可证实瓶颈。唯一父目录只占唯一路径的 21.62%，所以父目录解析复用的
理论 full-resolution 降幅为 78.38%。

原始 profile 位于
`E:\ZirconBuilds\frameworks06-doc-audit-profile-20260811.pstats`。此前一次 125 秒超时未稳定
复现，不作为基线。

随后三次非 profile 扫描通过不可变输入核验：2,867 个 `docs/**/*.md` 的 pre/post 指纹均为
`62aa19a0cbc42b143408394505b0da867dd3b027d0c684a7b37e031a99d49f3d`；三轮都扫描
2,315 份带有效 front matter 的文档与 73,862 条路径，违规数均为 512，有序违规投影指纹均为
`cdf2345656152fb695467e6fda0b52447fe5b5789f4c9dfd667bfb43f1eccb9a`。墙钟为
19.09 / 17.95 / 14.20 秒，p50 为 17.95 秒，max 为 19.09 秒。完整报告位于
`E:\ZirconBuilds\frameworks06-doc-audit-baseline-20260811.json`。

实现前的可丢弃内存原型在同一进程中先执行现实现，再执行三次候选算法。三次候选的完整 report
均与现实现逐项相等；full-resolution 操作从 16,500 降到 3,564，降幅 78.40%。同轮现实现
为 23.37 秒，候选三轮为 13.22 / 11.66 / 10.80 秒。该原型只验证方案方向，不进入仓库，
不替代测试先行、生产实现后的冻结输入测量或独立复审。

## 方案评估

### 采用：运行级 validator 与父目录解析缓存

1. 单次 `audit_document_paths` 创建一个路径 validator，继续以解析后的仓库根为唯一 authority。
2. 绝对 Windows drive/UNC 先 fail closed；远程 `scheme://` 仍在测试引用分类阶段排除；POSIX
   absolute 与仓库逃逸保持原诊断。
3. 对普通 leaf 只解析其父目录，并按 lexical parent 缓存最终路径；同目录下的文件复用一次
   final-path 查询。
4. leaf 自身若是 symlink 或 junction，必须解析 leaf，不能用父目录缓存绕过 reparse escape。
5. 缺失路径仍在验证其可解析父链未逃逸后报告 `missing path`，不能先返回 missing 而隐藏
   reparse parent escape。
6. 报告公开唯一声明数、full-resolution 总数、父目录解析数、leaf reparse 解析数和相对路径段
   解析数，使性能合同能以操作规模验证，不依赖脆弱的 CI 墙钟阈值。

### 拒绝：并行 full resolve

线程池仍保持每个唯一路径一次 full resolve，只把同一 O(U) I/O 压到更高瞬时并发，增加磁盘与
功耗压力，并使受共享机器争用影响更大；它不能作为首选结构修复。

### 拒绝：Git inventory 代替文件系统合同

`git ls-files` 或相似清单会把当前“实际仓库路径存在性”改成“tracked/untracked inventory”语义，
可能遗漏合法目录、忽略路径或工作区生成的受控输入。该方向属于合同改写，不是等价优化。

## 里程碑

### M0 基线与语义冻结

- 已保存当前完整有序违规投影与报告摘要到 `E:\ZirconBuilds`。
- 已在同一 docs 指纹下执行 3 次墙钟测量并记录 p50、max；full-resolution 调用数由 profile
  的 16,500 次和实现后的显式操作计数共同勾稽。
- 同父目录规模回归、相对路径段回归与 symlink/junction escape 回归已先行 RED，并在 validator
  实现后转为 GREEN；Windows 使用无需管理员权限的 junction 实际验证，不以 skip 旁路安全合同。

### M1 validator 实现

- 先把 `test_check_conventions.py` 中现有 document-audit 测试族迁入 folder-backed child；主测试
  模块只保留 unittest discovery 接线。删除原位置测试，不保留重复 owner 或 forwarding helper，
  并把主文件收回约 900 行以内后再增加性能回归。
- 引入单次审计生命周期的 parent-resolution cache，不建立第二份路径 authority。
- 保持现有错误字符串、排序、逐文档违规明细和 path cache 语义。
- 不增加 allowlist、compat shim、silent fallback 或平台专用成功旁路。

### M2 测量、回归与收口

- 新旧冻结输入的完整违规投影必须逐项相同。
- 31 项 Frameworks06 Python 契约、AST 与 diff-check 必须通过。
- 同父目录 synthetic scale 的 full-resolution 次数必须从 O(U) 收敛到
  `O(unique_parent + reparse_leaf)`；当前真实输入的 full-resolution 次数降幅目标至少 70%。
- 相同冻结输入至少 5 次 post 测量；p50 目标相对 M0 至少改善 30%，同时报告 max，不用单次
  最优值冒充结果。
- 完成独立二次审查后，才可进入 managed milestone validation 与协调器提交。

## 状态与产出记录

Python 3.11 reparse 修复后的同一输入参考比较逐项保留旧报告字段：2,871 份 `docs/**/*.md`
的 pre/post 指纹均为
`f5a485db44142d6724ec8f6b6adfe6bc1759668d6408b4e5dc88fb3b8ae806ec`，新 validator 与逐路径
full-resolve 参考实现的完整旧字段 report 相等。16,502 个唯一声明中，参考实现执行 16,499 次
full resolve，新实现执行 3,565 次，降幅 78.3926%。该轮共享树有 547 条外部 owner 路径债务；
语义等价与结构降耗不把全局 RED 改写成 GREEN。

current-source 五次 post 观测为 5.0486 / 5.3495 / 6.9385 / 6.3475 / 6.2924 秒，原始 p50/max
为 6.2924 / 6.9385 秒，但第 3/4 轮之间有外部 docs 漂移：有序投影、唯一声明与违规数随输入
从 547 变为 546，pre/post 指纹也不同。因此该轮明确拒绝，不能作为 performance acceptance；
Python 3.11 reparse 修复前的 accepted 样本也不冒充 current-source 证据。原始拒绝记录保存在
`E:\ZirconBuilds\frameworks06-doc-audit-post-python311-final-20260811.json`。实现已完成，31/31
static GREEN。fresh exact7 的内容复审 pre/post 指纹一致且结论 C0/I0/M0，但 closeout 输入审计随后
发现 snapshot 1615 没有包含 runner 必需且仍为 untracked 的 `tools/convention_exemptions.py`；按该
snapshot 提交会让 `tools/check_conventions.py` 导入不存在的模块，因此 snapshot 1615/exact7 已明确
拒绝，不得用于提交或 acceptance。候选范围扩为 exact8；其中 exemption inventory 已使用固定字符串
`allow` 预筛，并由 clean tracked/untracked 的跨行 `#[allow\n(...)]` 回归锁定，不恢复逐行
`allow(` 漏检。fresh exact8 内容与原子输入二次审查结论为 C0/I0/M0：Git inventory 对
tracked/dirty/untracked 生效并排除 ignored 文件，Git 失败 fail closed；lexical scanner 屏蔽
嵌套注释、普通/raw/byte/C 字符串与字符字面量，同时保留跨行 attribute 结构；workspace member
按最长 canonical root 归属。冻结墙钟 attestation 与 managed milestone closeout 等 coordinator
wakeup 后补齐，Session 不为外部 docs 漂移轮询或等待。

| 里程碑 | 状态 | 日期 | 证据 |
|---|---|---|---|
| M0 profiling | `complete` | 2026-08-11 | immutable p50/max 17.95/19.09 s；16,502 unique paths / 3,567 unique parents；热点为 Windows final-path resolution |
| M1 validator | `implementation_complete` | 2026-08-11 | folder-backed 测试 owner；parent-resolution cache；reparse/`..` full resolve；Python 3.11 reparse attribute 回归；31/31 static GREEN |
| M2 acceptance | `secondary_review_green / performance_attestation_pending` | 2026-08-11 | exact7/snapshot1615 rejected for omitted required owner；fresh exact8 C0/I0/M0；current-source old-field report equal；full resolve -78.3926%；5-sample input drift rejected；managed closeout pending |
