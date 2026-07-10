#!/usr/bin/env bash
set -euo pipefail

root="${1:-.codex/skills/superpowers}"
root_path="$(cd "$root" && pwd)"

printf 'superpowers/\n'
while IFS= read -r category; do
  count="$(find "$root_path/$category" -mindepth 1 -maxdepth 1 -type d | wc -l)"
  printf '  %s/ [%s]\n' "$category" "$count"
  while IFS= read -r item; do
    printf '    %s/\n' "$item"
  done < <(find "$root_path/$category" -mindepth 1 -maxdepth 1 -type d -printf '%f\n' | sort)
done < <(find "$root_path" -mindepth 1 -maxdepth 1 -type d -printf '%f\n' | grep -v '^scripts$' | sort)

printf '  scripts/\n'
while IFS= read -r script; do
  printf '    %s\n' "$script"
done < <(find "$root_path/scripts" -mindepth 1 -maxdepth 1 -type f -printf '%f\n' | sort)
