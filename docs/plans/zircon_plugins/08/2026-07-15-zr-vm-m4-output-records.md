# ZrVM M4 真实后端与 GC 产出记录

Plan: docs/plans/zircon_plugins/08-zr-vm.md
Milestone: M4
Status: completed
Files: ["Cargo.lock", "docs/plans/zircon_plugins/08/2026-07-15-zr-vm-m4-output-records.md", "docs/zircon_plugins/zr_vm_language/runtime.md", "docs/zircon_runtime/script/vm/gc_bridge.md", "zircon_plugins/zr_vm_language/runtime/Cargo.toml", "zircon_plugins/zr_vm_language/runtime/src/real_backend.rs", "zircon_plugins/zr_vm_language/runtime/src/real_backend/extension_host.rs", "zircon_plugins/zr_vm_language/runtime/src/real_backend/host_modules.rs", "zircon_plugins/zr_vm_language/runtime/src/real_backend/instance.rs", "zircon_plugins/zr_vm_language/runtime/src/real_backend/package.rs", "zircon_plugins/zr_vm_language/runtime/src/real_backend/runtime_owner.rs", "zircon_plugins/zr_vm_language/runtime/src/tests/real_backend.rs", "zircon_plugins/zr_vm_language/runtime/src/tests/support.rs"]

## Scope Delivered

- 新增 `ZrVmRuntimeOwner`，把 `ProjectSession`、native registrations 与 `Runtime` 的 unsafe Send/Sync 和销毁顺序收束到一个叶子 owner；所有调用和 Drop 均持有全局 ZrVM 锁。
- 补齐真实 `zr.zircon.extensions` native module，四个注册回调进入 `VmHostInterfaceRegistry` 的 capability gate 与 dense callback owner，不再依赖不存在的 facade。
- `ProjectSession::gc_step` 接入中立 `VmGcBudget`/`VmGcStepOutcome`；新增 cooperative policy fixture，通过 `VmPluginManager::gc_step` 验证调度链。
- 增加返回值 root 生命周期证明：真实 ZrVM string 降低为 `ScriptHostValue` 后，下一 collector step 的 cross-boundary count 为 0。
- `backend-zr-vm` feature 显式声明 binding 与 sys binding 两项依赖，根锁文件同步依赖闭包；默认 feature 不依赖 DLL。
- 更新 runtime plugin 与 GC bridge 模块文档，记录 owner、FFI panic boundary、预算和 root 语义。

## Fresh Testing Evidence

- `dc27faba132341b4ad8f98e84caa1377`：Windows、`backend-zr-vm`、`real_backend` filter，15 passed / 0 failed；doc-tests 0 failed。环境使用 `E:/Git/zr_vm/build/lib` 与 `E:/Git/zr_vm/build/bin`。
- `676e9eca55a449f99b779b0ae58eafd5`：Windows 默认 feature `validate-matrix -Package zircon_plugin_zr_vm_language_runtime -SkipBuild` 完整通过，Cargo test 与 doc-tests 均为 OK；默认构建不加载 ZrVM DLL。
- `9d48351478e04b7986309916507c0301`：owner-lock v2 feature 重编译进程正常退出；协调器在调用方超时后由活跃 PID 收口为 exit 0。功能执行结果仍以可审计的 `dc27...` 15/15 为主证据。
- scoped `rustfmt` 与 `git diff --check` 已通过，仅有仓库既有 LF/CRLF 提示。

## Review

- 独立复核：0 Critical / 0 Important；唯一文档关联索引问题已修复并复核关闭。

## 参考引擎与有意差异

- Zircon 当前 `VmPluginManager`/`VmGcBudget` 负责中立调度与诊断；插件只实现真实 collector adapter。
- ZrVM binding 负责 raw handle、GC root/cross-boundary 统计和 FFI `catch_unwind` trampoline；Zircon 不复制第二套 FFI ABI。
- Godot GDExtension 的 object/free/ref 生命周期用于校验显式 owner/drop 顺序；Bevy 的 panic containment 用于确认 unwind 必须在 FFI 边界截断。Zircon 的有意差异是把全部 ZrVM 实例串行化到进程锁，因为当前 binding runtime 是 process-global。

## 后续门禁

- 协调器执行 M4 review/commit；主计划 M4 状态由其 owner Session 同步为完成。
