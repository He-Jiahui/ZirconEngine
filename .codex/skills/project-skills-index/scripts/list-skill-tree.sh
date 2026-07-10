#!/usr/bin/env bash
set -euo pipefail

skills_root="${1:-.codex/skills}"
root_path="$(cd "$skills_root" && pwd)"

printf 'Skills root: %s\n\n' "$root_path"

while IFS= read -r -d '' skill_dir; do
  skill_folder="$(basename "$skill_dir")"
  printf '%s/\n' "$skill_folder"

  while IFS=$'\t' read -r child_name child_type; do
    suffix=""
    if [[ "$child_type" == "d" ]]; then
      suffix="/"
    fi
    printf '  %s%s\n' "$child_name" "$suffix"
  done < <(find "$skill_dir" -mindepth 1 -maxdepth 1 -printf '%f\t%y\n' | sort)

  skill_md="$skill_dir/SKILL.md"
  if [[ -f "$skill_md" ]]; then
    skill_name="$(grep -m1 '^name:' "$skill_md" | sed 's/^name:[[:space:]]*//' || true)"
    description="$(grep -m1 '^description:' "$skill_md" | sed 's/^description:[[:space:]]*//' || true)"

    if [[ -z "$skill_name" ]]; then
      skill_name="$skill_folder"
    fi

    if [[ -n "$description" ]]; then
      printf '  summary: %s | %s\n' "$skill_name" "$description"
    else
      printf '  summary: %s\n' "$skill_name"
    fi
  else
    printf '  summary: collection directory with no top-level SKILL.md\n'
  fi

  printf '\n'
done < <(find "$root_path" -mindepth 1 -maxdepth 1 -type d -print0 | sort -z)
