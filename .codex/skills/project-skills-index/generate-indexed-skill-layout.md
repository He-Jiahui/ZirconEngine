# Generate Indexed Skill Layout

- Use this file when creating or refactoring a project-local skill.
- Create a parent `SKILL.md` that acts as a concise navigation hub.
- In `.codex/skills`, every folder that groups child skills must have this parent `SKILL.md`; do not leave category folders as bare containers.
- Add short root-level instruction files whose filenames reveal the next decision or task branch.
- Move deeper instructions into child folders with their own `SKILL.md`.
- Use the parent `SKILL.md` to explain the child-skill categories and route the reader to the right child folder instead of forcing a full-tree scan.
- Keep the layout shallow enough that one command-line directory listing exposes the important branches.
- Exempt only resource/support folders such as `agents/`, `assets/`, `references/`, and `scripts/`.
- After creating or changing a skill, refresh `catalog-existing-skills/current-project-skills.md`.
- For the full template and writing rules, read `scaffold-indexed-skill/SKILL.md`.
