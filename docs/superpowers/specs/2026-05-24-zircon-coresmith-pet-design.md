# Zircon Coresmith Pet Design

## Purpose

Create a Codex-compatible custom pet based on the user's working style and current ZirconEngine context. The pet should read as a focused engine architect: calm, precise, builder/debugger oriented, and grounded in engine/runtime/editor work rather than a generic mascot.

## Context

The active workspace is ZirconEngine, a Rust engine/editor/hub project with heavy emphasis on runtime boundaries, Slint Hub UI, renderer data, asset pipelines, validation, and hard cutover discipline. Recent work and open files point to Hub build wiring, app bindings, responsive UI surfaces, and build verification logs.

## Approved Direction

Name: Zircon Coresmith

Concept: A compact precision-builder automaton with a faceted zircon core, graphite shell, small tool-like hands, and a calm inspector expression.

Style: 3D-toy / polished-sticker mascot. The form should be readable at pet size, with clean bevels, crisp silhouette, and a restrained engineering palette.

Rationale: This direction best matches the selected "focused engine architect" personality. It also carries the ZirconEngine identity through material, shape, and behavior without copying logos, text, UI, or repo-specific marks.

## Visual Identity

The pet has a dark graphite body, compact mechanical limbs, precise beveled panels, and a bright blue-cyan faceted crystal core in its chest. The face should be simple and steady: focused eyes, no exaggerated grin, no chaotic expression. The body should feel like a tiny inspector-builder that can sit beside a coding session.

Keep the palette controlled:

- graphite and muted charcoal for the body
- blue-cyan for the zircon core and small accent lights
- small off-white highlights for readability
- no readable text, logo marks, UI screenshots, code snippets, or symbols

The core, face, limb proportions, shell panels, and palette are identity locks that must remain consistent across all animation rows.

## Animation Contract

The final pet must use the current Codex pet atlas contract: nine rows, 192x208 cells, final atlas 1536x1872, and package files `pet.json` plus `spritesheet.webp`.

Rows:

- `idle`: subtle breathing, small blink, or tiny core pulse; calm and low-distraction.
- `running-right`: deliberate directional movement to the right with precise alternating steps.
- `running-left`: mirror `running-right` only if the crystal core, face, and body markings remain semantically correct when flipped; otherwise generate separately.
- `waving`: a small measured greeting with one tool-like hand or arm; no wave marks.
- `jumping`: vertical body motion only; no floor, shadow, dust, or impact marks.
- `failed`: low-power slouch or dimmed core with attached, opaque state detail if needed; no floating symbols.
- `waiting`: expectant approval/help pose, distinct from idle and review.
- `running`: focused task-processing posture, such as leaning into work or internal computation; not literal sprinting.
- `review`: close inspection pose with head tilt or focused lean; no papers, magnifiers, UI, punctuation, or new props.

## Generation Approach

Use the hatch-pet workflow with the image generation skill as the only visual generation layer. Prepare a run from text only, using the approved concept as `pet-notes`, style preset `3d-toy`, and no brand discovery because this is a personal/workstyle pet rather than an external brand.

Data flow:

1. Prepare the pet run manifest and prompts.
2. Generate the base image as the canonical identity reference.
3. Generate row strips grounded by the canonical base and layout guide for each state.
4. Copy each selected generated output into the run's decoded paths and mark jobs complete.
5. Derive `running-left` only if `running-right` is visually safe to mirror.
6. Extract frames, inspect components, compose the atlas, validate it, render contact sheet and previews.
7. Package the final files under the local Codex pets directory.

## Error Handling And QA

Reject and repair any row with identity drift, missing frames, copied guide marks, shadows, glows, detached effects, cropped bodies, visible chroma backgrounds, wrong facing direction, stagnant animation, or row semantics that do not match the state.

If a row generation fails at transport level, retry once with its retry prompt and the same required inputs. If deterministic extraction produces size popping while the source strip is stable, rerun extraction with the hatch-pet stable-slot method before regenerating art.

Final acceptance requires:

- atlas validation passes
- frame inspection has no errors
- contact sheet is visually consistent across all nine rows
- preview GIFs show suitable motion without unintended scale jumps
- `pet.json` and `spritesheet.webp` are packaged together

## Out Of Scope

Do not integrate this pet into ZirconEngine source code. Do not modify Hub, runtime, editor, or plugin code. Do not create logos, repo branding, UI icons, or visible text inside the pet artwork.
