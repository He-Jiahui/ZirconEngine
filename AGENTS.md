# ZirconEngine working agreement

## Scope and authority

- Follow the user's current task and standing authorization. Complete the authorized scope, including relevant review, validation, and fixes. Honor explicit audit-only, plan-only, and approval checkpoints.
- Resolve routine implementation choices from the request, current code, and repository conventions. Ask only for a consequential missing decision or permission; continue independent authorized work.
- These project rules and skills operate within system, developer, and user instructions. A skill cannot expand the task, grant external-action permission, or override a user-authorized exception.
- Committing, pushing, publishing, and sending external notifications require authorization for those actions. Completing a milestone alone does not grant it.

## Context and evidence

- Load only skills relevant to the task. Reuse already-read, unchanged context; reopen it when changes or missing details matter.
- Reuse verification and review evidence for the same relevant source, tests, configuration, and environment. Re-run affected checks after relevant changes or failures. Pending receipts and dry runs are not passing validation.
- Use [the validation policy](docs/testing-and-validation/milestone-validation-policy.md) for test scope and cadence. Small documentation changes need structural checks, not Rust builds or invented unit tests.
- Before every compilation-capable command (`check`, `build`, `test`, including `--no-run`), explicitly tell the user which engine modules, Cargo packages, features, targets/test filters and profile will run, with the scoped command and coverage reason. Build only the smallest affected module/dependency set, with default features disabled and needed features named. Use `tools/dev/dev-module-build.ps1` with an explicit `-Package` for supported selections. Full builds are prohibited: no `--workspace`, `--all`, `--all-features`, `--all-targets`, implicit root-wide selection or equivalent all-package loops/matrices, including wave/release and root manifest/lockfile/toolchain work. Unknown scope requires dependency inspection, not a full build. Follow [the build declaration](.codex/skills/zircon-dev/validation/guide.md#declare-every-build). Measure final engine packages, symbols, shared build products and incremental caches separately.
- Use [the engineering entry](.codex/skills/zircon-engineering/SKILL.md) for delivery and priority, and [Zircon Dev](.codex/skills/zircon-dev/SKILL.md) for Rust work. Select specialist guidance as needed.

## Shared checkout and tooling

- Preserve foreign and preexisting changes. Follow [the checkout policy](.codex/skills/zircon-dev/references/main-branch-development-policy.md); an explicitly authorized branch or worktree takes precedence over the default.
- Inspect active chats, current diffs, and relevant plan/handoff notes when overlap is plausible. Preserve other tasks' changes and use explicit scope handoffs; do not register, heartbeat, claim paths, or submit work through the retired local coordinator.
- The old local coordinator and its validation workflow are retired. Do not start or restore its service, tray, worker, scheduled tasks, automatic synchronization, or queue. Session registration, leases, attribution APIs, and coordinator commit/closeout APIs are no longer prerequisites. Preserve data and follow [retirement and recovery](docs/tooling/coordinator-retirement.md). Use the independent Jenkins coordinator through [jenkins-coordination](.codex/skills/jenkins-coordination/SKILL.md); a verified `.codex/state/jenkins-coordinator/active.json` with `enabled` and `functionalTestsAllowed` permits coordinator functional tests and bounded command submission within the task's authorization. Whole-workspace, milestone and production migration acceptance remain separate gates. `tools/dev/local-cargo.ps1` provides independent local command evidence with physical output checks; it does not grant milestone or Jenkins acceptance.
- Preserve source attribution, shared-index ownership, live process identity, historical receipts, and actual acceptance gates during retirement. Do not erase the old database, queued work, artifacts, or locks to simulate completion.
- All compilation products and compiler caches must physically stay under the drive-root `D:\cargo-targets`, `E:\cargo-targets`, or `F:\cargo-targets` directory. Local tooling, build scripts, CI, and skill examples must reject every other location, including `C:`, repository-local `target`, `D/E/F:\targets`, `D/E/F:\ZirconBuilds`, nested lookalike roots, and path aliases.
- For asynchronous work, follow [receipt lifecycle](.codex/skills/zircon-project-skills/cross-session-coordination/references/receipt-lifecycle.md). Continue independent work, reconcile the existing request, and keep pending acceptance open.

## Skill maintenance

- `.codex/skills` is the source of project skill content. Maintain its discovery entries and generated Codex catalog through [project-skills-index](.codex/skills/project-skills-index/SKILL.md).
- Only discoverable skills use `SKILL.md` with valid metadata. Category indexes and supporting guides use ordinary Markdown.
- Rules within reference repositories under `dev/` and nested worktrees apply to their own trees. Inspect their local guidance when working there; do not apply it to unrelated ZirconEngine files.

## Execution permissions and workflow ownership

- The user explicitly authorized Full Access on 2026-10-18. `.codex/config.toml` selects `default_permissions = ":danger-full-access"` with `approval_policy = "never"`. Do not require `zircon-strict`, Windows sandbox setup or sandbox ACL repair for this selection. The retained `zircon-strict` allowlist is an optional restricted profile; select it only when the user asks.
- Full Access changes command execution permissions. Jenkins still owns Git, branches, worktrees, source preparation, generation, compilation, validation, artifact reuse, integration and cleanup under the user's standing instructions. Preserve other chats' changes, unknown source and artifacts, Git metadata and historical receipts; access alone does not authorize unrelated changes or deletion.
- Existing chats can retain an explicit older selection. Follow [activation and validation](docs/tooling/codex-file-permissions.md), use the supported desktop permission selection for those chats, and verify actual execution before claiming recovery. Do not edit the chat database or treat a saved default as proof that every active chat changed.
