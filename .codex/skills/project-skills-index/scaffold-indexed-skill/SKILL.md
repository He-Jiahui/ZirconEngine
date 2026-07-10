# Scaffold Indexed Skill

## Progressive Disclosure Index

- If you need a copyable layout example, read `layout-template.md`.
- If the target skill is small enough to stay flat, stop and keep a single top-level `SKILL.md` instead of forcing extra nesting.

## Workflow

- Start by deciding whether the target skill truly needs chunking. Do not add child folders for tiny skills.
- Create the parent `SKILL.md` as a short navigation hub with a "Progressive Disclosure Index" near the top.
- In repository-local `.codex/skills`, any folder that exists to classify child skills must have this parent `SKILL.md`. A bare grouping folder is incomplete.
- Add short root instruction files when their filenames can advertise distinct branches such as discovery, validation, or generation.
- Create child folders when a branch needs more than a short note. Put a child `SKILL.md` inside each folder and let that file point to any deeper branch files.
- Keep the important branches visible from a shallow listing of the skill root.
- Prefer one extra level of child folders. Justify any deeper nesting in the parent `SKILL.md`.
- Exempt only support folders such as `agents/`, `assets/`, `references/`, and `scripts/`; they are not child skills and should not be treated like category nodes.
- After scaffolding the new skill, refresh the project catalog in `../catalog-existing-skills/current-project-skills.md`.
