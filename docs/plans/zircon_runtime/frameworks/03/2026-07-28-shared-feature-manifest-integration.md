# Frameworks03 shared feature manifest integration

Plan: docs/plans/zircon_runtime/frameworks/03-optional-features-and-profile-matrix.md
Milestone: M1
Status: completed
Files: ["Cargo.lock", "docs/plans/zircon_runtime/frameworks/03/2026-07-28-shared-feature-manifest-integration.md", "zircon_runtime/Cargo.toml"]

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M1 | `text` 直接图形依赖归属与 export validator 文件身份依赖 | completed | 2026-07-28 | 当前共享输入 `Cargo.lock` SHA-256 `a0d67a7b2f62afe3e2ff3347ad2a195da168fc1bc7470ac0365c0bc8b6f6fce6`、`zircon_runtime/Cargo.toml` SHA-256 `ab49b279f38d05767291127d9808735642d83b8fe2932f3b0dbd9a04587138f1`。 |
| M1 | testing：共享 feature/dependency 当前源门 | passed | 2026-07-28 | Text-only managed check `dc782862a3fb46d8a8f1f045a4c3e1b1` / `f8fbf365e79441ccbc6f46359b482aed` exit 0；Plugins09 managed bin gate `49b906fe060e4b88b28dc134c171f153` / `9c89b92bdda047bc8e67d631002e6d88` 为 9/9，integration/scale gate `fd95f01579e448d39126786ac1faa2af` / `27edc33ca331441296da06c1f2a53181` 为 2/2；Frameworks03 静态矩阵 3/3。 |

## Scope Delivered

本切片只收敛共享 Cargo feature/dependency 真相：`text` 显式拥有生产代码直接使用的
`glyphon`、`naga` 与 `wgpu`，`graphics` 继续通过既有 `text` feature 组合获得它们；
export validator 显式声明 `same-file`，用于保留报告输出与内容 artifact 的文件身份冲突判定。
没有增加旧 feature alias、兼容 shim、隐式依赖或重复 owner。

该记录将两个已分别通过当前源受管门、但因共享整文件提交边界无法由任一 stale 业务会话
单独提交的 manifest 变更，重新归属到 Frameworks03 的 feature 矩阵 owner。它不宣称
Text01 的五个开放 failure 已修复，也不替代 Plugins09 的 failure closeout。

## Fresh Testing Evidence

- Text-only managed check `dc782862a3fb46d8a8f1f045a4c3e1b1` / `f8fbf365e79441ccbc6f46359b482aed`：exit 0。
- Plugins09 managed bin gate `49b906fe060e4b88b28dc134c171f153` / `9c89b92bdda047bc8e67d631002e6d88`：9/9。
- Plugins09 integration/scale gate `fd95f01579e448d39126786ac1faa2af` / `27edc33ca331441296da06c1f2a53181`：2/2。
- `python tools/tests/test_frameworks_03_domain_feature_matrix.py`：3/3。

## Review

Exact3 successor snapshot `1180` 已完成控制器静态审阅；受管里程碑独立 review 记录在提交门中。
