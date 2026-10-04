# ZirconEngine working agreement

## Scope and authority

- Follow the user's current task and standing authorization. Complete the authorized scope, including relevant review, validation, and fixes. Honor explicit audit-only, plan-only, and approval checkpoints.
- Resolve routine implementation choices from the request, current code, and repository conventions. Ask only for a consequential missing decision or permission; continue independent authorized work.
- These project rules and skills operate within system, developer, and user instructions. A skill cannot expand the task, grant external-action permission, or override a user-authorized exception.
- Committing, pushing, publishing, and sending external notifications require authorization for those actions. Completing a milestone alone does not grant it.

## Context and evidence

- Load only skills relevant to the task. Reuse already-read, unchanged context; reopen it when changes or missing details matter.
- Reuse verification and review evidence for the same relevant source, tests, configuration, and environment. Re-run affected checks after relevant changes or failures. Pending receipts and dry runs are not passing validation.
- Use [the validation policy](docs/plans/milestone-validation-policy.md) for test scope and cadence. Small documentation changes need structural checks, not Rust builds or invented unit tests.
- Use [the engineering entry](.codex/skills/zircon-engineering/SKILL.md) for delivery and priority, and [Zircon Dev](.codex/skills/zircon-dev/SKILL.md) for Rust work. Select specialist guidance as needed.

## Shared checkout and tooling

- Preserve foreign and preexisting changes. Follow [the checkout policy](.codex/skills/zircon-dev/references/main-branch-development-policy.md); an explicitly authorized branch or worktree takes precedence over the default.
- Inspect active chats, current diffs, and relevant plan/handoff notes when overlap is plausible. Preserve other tasks' changes and use explicit scope handoffs; do not register, heartbeat, claim paths, or submit work through the retired local coordinator.
- The old local coordinator and its validation workflow are retired. Do not start or restore its service, tray, worker, scheduled tasks, automatic synchronization, or queue. Session registration, leases, attribution APIs, and coordinator commit/closeout APIs are no longer prerequisites. Preserve data and follow [retirement and recovery](docs/tooling/coordinator-retirement.md). Use the independent Jenkins coordinator through [jenkins-coordination](.codex/skills/jenkins-coordination/SKILL.md); a verified `.codex/state/jenkins-coordinator/active.json` with `enabled` and `functionalTestsAllowed` permits coordinator functional tests and bounded command submission within the task's authorization. Whole-workspace, milestone and production migration acceptance remain separate gates. `tools/dev/local-cargo.ps1` provides independent local command evidence with physical output checks; it does not grant milestone or Jenkins acceptance.
- Preserve source attribution, shared-index ownership, live process identity, historical receipts, and actual acceptance gates during retirement. Do not erase the old database, queued work, artifacts, or locks to simulate completion.
- All compilation products and compiler caches must physically stay under the drive-root `D:\cargo-targets`, `E:\cargo-targets`, or `F:\cargo-targets` directory. Local tooling, build scripts, CI, and skill examples must reject every other location, including `C:`, repository-local `target`, `D/E/F:\targets`, `D/E/F:\ZirconBuilds`, nested lookalike roots, and path aliases.
- For asynchronous work, follow [receipt lifecycle](.codex/skills/zircon-project-skills/cross-session-coordination/references/receipt-lifecycle.md). Continue independent work, reconcile the existing request, and keep pending acceptance open.

## Skill maintenance

- `.codex/skills` is the source of project skill content. `.opencode/skills` contains generated adapters; edit the source and run the [synchronizer](.codex/skills/project-skills-index/SKILL.md).
- Only discoverable skills use `SKILL.md` with valid metadata. Category indexes and supporting guides use ordinary Markdown.
- Rules within reference repositories under `dev/` and nested worktrees apply to their own trees. Inspect their local guidance when working there; do not apply it to unrelated ZirconEngine files.

## File write boundary

- The canonical file-write allowlist is `[permissions.zircon-strict.filesystem]` in `.codex/config.toml`; use the `zircon-strict` permission profile for this checkout. A readable input does not authorize creating, copying, pasting, extracting, modifying, moving, renaming or deleting files there. Every destination, temporary file, cache and child-process output must match an explicit write grant.
- Keep control configuration, hooks, Git data, reference trees and unlisted directories read-only. Do not expand the allowlist, switch to Full Access, use an unsandboxed service, or stage files outside permitted paths to bypass a denial. Permission maintenance requires the user's explicit authorization for that change.
- Existing chats can retain older permissions; saved configuration and a successful setup receipt do not prove that the current chat is restricted. Follow [activation and validation](docs/tooling/codex-file-permissions.md), and do not claim enforcement until the selected profile and real file-operation probes have been verified.
