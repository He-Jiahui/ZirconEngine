# Runtime10 / Runtime11 镜像执行代际：独立下层回归证据

## 归属与验收状态

关联 [Runtime10 DLL worker 生命周期 failure](failure-2026-08-10-dynamic-runtime-dll-unload-worker-lifetime.md) 和 [Runtime11 completion backpressure failure](../11/failure-2026-07-22-asset-worker-shared-completion-backpressure.md)。两项仍为 open。

本批次验证未安装的镜像执行代际候选中的逻辑准入、执行 pin 计数、构造与最终 Session 退休原子性、关闭期限和重开权限。真实 Runtime 消费者接线、物理 worker join、宿主 Library 和 DLL 生命周期验收继续等待。

## 冻结输入与实际执行

- 输入共 13 个：9 个候选 Rust 源文件、独立 probe 的 Cargo.toml/Cargo.lock/lib.rs，以及现行 local Cargo wrapper。
- [13 输入封印](../../../../../.codex/tmp/failure-roll-20261002-runtime10-runtime11-implementation-preparation/v2/standalone-probe-sealed-manifest.json) SHA256 `5736f373b74cf83a40e69663abe423f5d0cdf9d22ec671568977e4dfd6402b67`。
- Windows 原生 Rust / Cargo 1.94.1，x64 MSVC 环境，Cargo 保持 `--locked`。实际子进程直连环境记录 `NO_PROXY=*`，非空 proxy keys 为 0。
- 实际 Cargo PID `28136`，创建 FILETIME `134354716099622683`，映像和 SHA 绑定在 observer 收据中。
- 实际执行从 `2026-10-03T03:26:48.579405+00:00` 到 `2026-10-03T03:27:02.930475+00:00`（UTC）。

实际 cwd：`D:\cargo-targets\zircon-local\windows\failure-roll-20261003-image-generation-lower23-v3\source`。

```text
cargo +1.94.1 test --locked --lib --manifest-path D:\cargo-targets\zircon-local\windows\failure-roll-20261003-image-generation-lower23-v3\source\probe\Cargo.toml --target-dir D:\cargo-targets\zircon-local\windows\failure-roll-20261003-image-generation-lower23-v3\target -- --test-threads=1
```

源码、副本、Cargo 配置、选定工具的执行前后检查通过。编译产物、build、Cargo home、sccache 和 TEMP/TMP/TMPDIR 均在物理 `D:/cargo-targets` 下；七个精确目录保存于实际 observer 收据。

## 结果与回执

实际运行 **23 passed / 0 failed / 0 ignored / 0 filtered out**，23 个预期测试名全部匹配。Cargo、wrapper、Root 启动器均 exit 0，所有自有子进程终态已核对，pending/unknown 为 false。

- [Root 终态](../../../../../.codex/tmp/failure-roll-20261003-image-generation-lower23-v3/terminal.json) SHA256 `a2a90de1751602d7e65cb9e9a3973b7bef2e6545b33ef3bca21dbb8ad4e1b6db`。
- [实际 Cargo observer](../../../../../.codex/tmp/failure-roll-20261003-image-generation-lower23-v3/actual-cargo-observer.json) SHA256 `e7a3de303c4f1f719e4a99b4102a3d4204364090e5bd7a7d6bcdd35431184338`。
- [测试原始日志](../../../../../.codex/tmp/failure-roll-20261003-image-generation-lower23-v3/test.log) SHA256 `09bc346ad399be5c80fc81e6b8ba0d6d4b7a7cfcc1907d66ebcf055a6b0c1e4c`。
- [启动器独立审查](../../../../../.codex/tmp/failure-roll-20261003-generation-lower23-v3-independent-review/review.json) SHA256 `a4713492b5a7fbad6e0a7440bcbe6234a3b38b50a7110590246675b186c75c21`，C/I/M 0/0/0。

第一次实际尝试 exit `3221225794`（`0xC0000142`），目标测试执行数 0；该历史失败和故障诊断保持原状。当前尝试成功没有建立该历史 DLL 初始化故障的根因。

## 后续依赖与关闭条件

1. 将已验证逻辑接入真实构造、Session 注册/销毁、worker、timer、dispatcher、native provider 和 callback 所有权链，重新核对现行源码及直接消费者。
2. 验证物理 worker join、关闭超时后保留 pin/slot、重试和完整静默、last-image-release 与重新加载；逻辑静默收据单独不能证明物理 join。
3. 执行两个 canonical failure 的原始复现、上层验收和所需规模/性能批次，完成独立审查，再分别回传、精确提交与按 SHA 去重通知。

本记录没有生成 fixed 文件、关闭 lifecycle、安装候选或取得 commit SHA。Jenkins、产品、DLL 和整个工作区验收继续按各自条件执行。
