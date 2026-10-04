---
date: 2026-10-04
implementation_status: source_applied
validation_status: native_validation_pending
performance_status: original_acceptance_open
---

# 任务执行归属与场景加载拒绝

| 范围 | 状态 | 证据 |
| --- | --- | --- |
| Core、场景加载与三个调度器出生点，共 18 个源文件 | `source_applied_native_validation_pending` | 冻结清单、非作者源码审查、实际应用收据；格式检查通过，类型检查、测试与性能尚未通过 |

指定 scope 的旧加载路径改用目标图的 Compute 池；修复后使用传入调度器的真实池及原图生命周期。原图的关闭原因先于目标 scope 的关闭原因判定，完成对象保留原图身份。目标 scope 准入与关闭共用原锁，取消作用于原执行器中的真实任务，终态发布后退出 scope 账本。

加载入口返回 typed refusal。Reload queue 在调度成功后才发布 pending 元数据、顺序项和计数；拒绝携带原始错误进入帧失败报告。读写服务和 VM 包发现保留原有独立诊断计数。

原测试未删减；新增回归覆盖占满原执行池后的执行归属、停止与 scope 关闭优先级、外国 scope 关闭取消、加载拒绝及队列不发布。源码审查不替代编译、运行、物理堆与产品性能验证。

本轮实际证据位于 `E:/cargo-targets/zircon-local/source-repairs/current-main-20261004/`：

- 冻结清单：`core-loader-owner-fix-v2/manifest.json`，SHA256 `b0fdf81cea55d0ec73386f34336cd4cc9b6d2eec8266a282f6ddff7fe5381fc1`。
- 补丁：`core-loader-owner-fix-v2/candidate.patch`，SHA256 `6fec278454ff6ebf0d0dc9ad199adf44676664aa732a06c16686d31aa5a41dd6`。
- 独立源码审查：`core-owner-review-v2/review.json`。
- 实际应用：`core-loader-owner-fix-v2/actual-guarded-source-apply.json`；18 个文件写入，配置、根 manifest／lock 和 Git 索引应用前后哈希相同。

后续检查、正常回归及隔离 Release 验证按批次启动；启动后继续 Editor 功能修复，结果留待后续源码里程碑离散核验。原性能阈值、规模和完成标准保持，尚未宣布性能达标。
