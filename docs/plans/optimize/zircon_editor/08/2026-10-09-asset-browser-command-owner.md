# Editor08：Assets 命令入口与原资产效果 owner 对齐

日期：2026-10-09。状态：已接入源码；native 测试、产品验收和性能达标尚未确认。

## 实际缺口与修复

当前 Unreal 工具栏声明 `view.asset_browser.open`，但同一 `Workbench/OpenAssetBrowserFromToolbar` Click binding 仍绕到旧 typed AssetCommand。只改 binding 又会丢失 `AssetPreviewRefreshRequested`：通用 canonical descriptor 原先只发送 OpenView。

本次修改两个现有生产 owner：工具栏 binding 进入已有 EditorCommand registry；原命令 descriptor 为该 canonical ID 发出原 EditorAssetEvent，继续使用原资产 handler。Always、contexts、capabilities、菜单位置和其他 view descriptor 保持原合同。原 handler 的打开/聚焦视图、layout/presentation、visibility 和资产预览刷新继续保留。

旧 main menu/overflow 的 `AssetSurface/OpenAssetBrowser` 是另一 key，其 typed binding 不变。本次没有创建新 command owner、别名或平行刷新路径。

## 原测试与真实回归

原 same-key 单元测试同步改为 canonical EditorCommand payload，保留原 test 名、精确 key 和 Click path。原 10 个直接测试继续激活，并增加 1 个真实 context/event owner case。

现有 retained toolbar 回归通过 EventRuntimeHarness、manager/service 和原 handler 派发，检查真实 journal 的 asset event/canonical operation/binding path、AssetPreviewRefreshRequested、host preview refresh、layout/presentation、opened-view visibility 和 MRU。尚未实际运行这些 native 用例。

## 已取得证据

- 独立核心审查：v1 的旧同 key payload 断言 P2 已在 v2 修复。v2 source review accepted；native pending。
- v2 patch SHA-256：`4d25a3fd3f5604078eb293c2ff5f3382ec0bd4d8c57a51d6fd01f05d09a964b8`。
- 实际接入凭据：`E:/cargo-targets/zircon-local/source-repairs/current-main-20261008/editor-asset-browser-command-owner-v2/actual-canonical-apply.json`；SHA-256 `3d2b651bce4596f6f8388474a39bdaf44dd6f45ae7aa9959223aad0994c03c45`。
- 4 文件实际 git apply、rustfmt --check、git diff --check 均 exit 0；精确 LF postimages 匹配。Git 写入产生的 CRLF raw hashes 单独记录。
- AGENTS、权限配置、Cargo 配置/lock 和共享 index 在本次接入前后相同；其他来源修改保留。

## 异步验证与剩余门槛

先前 request `runtime-editor-source-repair-batch-20261009-02` 的真实终态是三包 Cargo check exit 101，后续 11 命令未调用；该 request 的 sourceUnchanged=true 不证明本次新源码已验证。

下一批会组合原 Runtime/Editor/App 合同、实际 toolbar/registry cases 和原 Release 性能 producer。批次草稿尚未 dispatch；共享目录 8 个 UI caller 被其他任务更新后，旧 59-case 候选绑定已失效，必须使用当前源码的实际 case/name/hash，不覆盖其他任务的测试。模块可达性、popup owner、产品表现另有真实门槛待验证。

本项是恢复真实功能入口的修复，没有测得 CPU、allocation 或 P95 收益。Editor08 原 >=20% helper、Editor386 原 >=40% helper，以及完整目录/Unicode/owner churn/native 产品要求仍保留。

对应[计划完成列表](../../../astra/features/editor/2026-10-09-editor08-asset-browser-command-completion-list.md)。
