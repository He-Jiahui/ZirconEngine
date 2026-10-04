---
name: project-skills-index
description: Maintain the Codex repository skill library, its discovery entries, and generated catalog.
---

# Project Skills Index

`.codex/skills` is the canonical source. This library is maintained for Codex. Independently useful workflows use `SKILL.md` with valid `name` and `description`; categories use `index.md`, and supporting procedures use `guide.md` or references.

## Keep discovery small

- Give each distinct workflow one entry in [the catalog](catalog-existing-skills/current-project-skills.md). Reuse an existing owner before adding a skill.
- Put a policy, technique, operating mode, or checklist in that owner's guides. A numbered plan or another rule does not by itself need a new skill.
- Descriptions state concrete triggers. Avoid overlapping triggers covering every feature, failure, or completion.
- Link each policy's authority instead of copying validation, ownership, permissions, or acceptance rules into every helper.
- Select references for the current operation. A parent tour, all child guides, extra approval, or delegation is not required merely because a skill was selected.
- Preserve supported metadata and invocation policy unless the requested change includes them.

Reference collections: [engine guides](../zircon-project-skills/index.md) and [development techniques](../superpowers/index.md). For a new layout, use [scaffolding](scaffold-indexed-skill/index.md) and [the template](scaffold-indexed-skill/layout-template.md).

## Refresh the catalog

After changing discovery entries, use an available Python interpreter with PyYAML:

```powershell
python -B .codex/skills/project-skills-index/scripts/refresh_catalog.py --write
python -B .codex/skills/project-skills-index/scripts/refresh_catalog.py --check
```

The helper reads source frontmatter and writes only the generated Codex catalog. It does not manage another client, register a session, or require a service. If `python` resolves to an unavailable Windows alias, use the existing `.jenkins/runtime/python/python.exe` interpreter.

Validate actual skills with the current skill-creator validator, check local routes, and run helper checks when the helper changes. Skill prose maintenance needs structural checks instead of Rust builds.
