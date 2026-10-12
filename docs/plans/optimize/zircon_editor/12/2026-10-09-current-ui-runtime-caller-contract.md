# 当前 UI 资源调用合同修复

日期：2026-10-09。状态：11 个测试源码文件已实际接入；原生测试、资源可达性和性能验收尚未完成。

## 原因和修改

当前 Unreal 工作区已迁移到现有资源 owner，部分旧测试仍引用退役的 shell、toolbar 和组件路径。沿真实 `FileCache::load_store`、模板注册与导入链路修正这些调用，并按当前资源的 kind、root、节点和 import 检查合同。

`main_band` 保持 View 中的节点身份。原生组件样例使用现有 Showcase 和共享组件目录，覆盖真实四个 prototype section 与 57 个原生类型。命令治理继续核对 21 个 canonical asset/action/binding 身份，保留五个退役命令别名的拒绝检查。

本次共修改 11 个测试文件，保留原有 47 个活跃用例及强可达性断言。当前外部任务的 raw palette 和 26/48/32 几何合同保持原样；未采用此前 557 处 token 重写候选。

## 已有证据

- 独立 source review：`accepted_source_native_pending`，无未解决 findings。
- 接入前核对 273 个依赖、全部源码前像、补丁和独审凭据；接入后核对 11 个 normalized LF 后像。
- `git apply --check`、实际 apply、Rust 2021 格式检查与 scoped diff 检查均 exit 0。
- 本次接入期间控制文件和共享 Git index 的 hash 未变化；没有创建源码副本或执行 native job。
- 原 raw 267 / Fixed 5262 上限仍保留；当前 source inventory 为 101 / 3893。这是静态治理数据，不是动态主题或产品性能结果。

候选补丁 SHA-256：`0b1f09e35c9b8a775c151b8195b1aff0bbc659c3406ce982ca0f8c0f0f3e11ea`。

独审凭据：`E:/cargo-targets/zircon-local/source-repairs/current-main-20261008/editor-ui-runtime-caller-core-review-v1/review.json`，SHA-256 `7c02a3fdb92053029c599432e6bea9ac4c3617ca8393e1714f48d2601453fe56`。

实际接入凭据：`E:/cargo-targets/zircon-local/source-repairs/current-main-20261008/editor-ui-runtime-caller-contract-fix-v2/actual-canonical-apply.json`，SHA-256 `39f86c7e22d48033daad71eaa22a5596a39a5ebc2685c74117d7ec6dd03953f1`。

## 验收仍开放

强可达性门按真实 support helper 仍有 96 个未被 root imports 到达的 Component：258 个文档中有 205 个 Component、110 个被引用 locator。原测试和 `offenders.is_empty()` 保留；未增加 root、注册豁免或降低断言。

下一批一次纳入这 47 个合同用例、实际模块挂载/绘制/命中回归、Details 只读合同修复以及原 Release producers。启动后继续独立功能修复，在实质源码或证据检查点读取匹配 request 的终态凭据；不等待或持续监控编译。

Batch03 已因 `Cargo.lock` 和 `zircon_runtime/Cargo.toml` 前像变化在启动前拒绝，Cargo 未运行，15 项命令均未执行。其历史凭据不作为本次修复通过证明。原 Release 阈值、完整矩阵及 Jenkins/产品验收保持开放。

完成列表见 [Editor UI 调用合同完成列表](../../../astra/features/editor/2026-10-09-editor12-current-ui-runtime-caller-completion-list.md)。
