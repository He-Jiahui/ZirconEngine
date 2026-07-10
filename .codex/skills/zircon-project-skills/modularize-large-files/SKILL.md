---
name: modularize-large-files
description: Use when editing or adding code in `zirconEngine` and a touched source file is approaching roughly 1000 lines, already exceeds that size, or is about to gain another responsibility. Apply it when the easy path is to keep stacking logic into one file instead of extracting coherent modules.
---

# Modularize Large Files

## Overview

Keep implementation files around 1000 lines when practical. Treat that threshold as a design warning: once a touched file is near or above it, default to splitting by responsibility instead of appending more logic.

## Quick Check

- Check line count early for any file receiving substantial new logic.
- PowerShell: `(Get-Content <path> | Measure-Object -Line).Lines`
- WSL: `wc -l <path>`
- Use this threshold:
  - Under roughly 900 lines: small edits are fine; do not add a second unrelated responsibility.
  - Roughly 900-1100 lines: plan the split if the change adds new helpers, subsystems, or protocols.
  - Above roughly 1100 lines: modularization is the default unless the user explicitly tells you not to.

## Splitting Rules

- Split by responsibility, not by suffixes like `part1`, `part2`, `misc`, or `helpers2`.
- Extract cohesive units such as parsing, diagnostics, JSON, transport, symbol resolution, formatting, codegen, or CLI wiring.
- Keep the original file as the orchestration boundary only if it still has one clear purpose after extraction.
- Move related declarations, static helpers, and tests together when they form one responsibility.
- Update build files, headers, includes, and registrations in the same change.

## Naming Guidance

- Prefer semantic names like `stdio_transport.c`, `stdio_json.c`, `parser_diagnostics.c`, or `compiler_symbols.c`.
- Avoid generic catch-all files like `common_utils.c` unless the code is genuinely cross-cutting and still cohesive.
- If a new module introduces public surface, give it the narrowest header or API that serves current callers.

## When Not to Force a Split

- Generated files, vendored code, or machine-produced tables.
- Large single-purpose files where splitting would only create artificial churn and weaker boundaries.
- Cases where the user explicitly asks to keep layout unchanged.
- In these cases, state why the file stayed whole and still avoid piling on unrelated responsibilities.

## Completion Standard

- Do not call the refactor complete if a touched file crossed the threshold only because new code was stacked into an already oversized file.
- If you defer the split, record the concrete reason and the smallest follow-up boundary that should be extracted next.
- Prefer making the split in the same change that introduced the new responsibility.

## Example Signals

- A parser file now contains lexing helpers, error formatting, and compile-time evaluation.
- A transport file now contains protocol parsing, JSON encoding, request routing, and lifecycle state.
- A test file has grown to cover multiple subsystems with unrelated fixtures and helpers.
- When you see this shape, stop adding code and cut a module boundary first.

## Red Flags

- "Just one more helper in this same file."
- "I will split it after this feature lands."
- "The file is big, but everything kind of belongs here."
- "I will add a misc section at the bottom."
- All of these mean: re-evaluate the boundary and split first.
