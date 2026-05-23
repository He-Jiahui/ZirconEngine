# Zircon Neko Pet Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create and package the Codex custom pet `Zircon Neko` as a validated anime chibi catgirl animated atlas.

**Architecture:** This is an asset-generation workflow, not a ZirconEngine code change. The hatch-pet scripts own deterministic atlas geometry, frame extraction, validation, contact-sheet generation, preview rendering, and packaging; the image generation skill owns all visual generation.

**Tech Stack:** Codex hatch-pet skill, built-in image generation skill, Python hatch-pet scripts, PowerShell, JSON manifests, WebP sprite atlas, local Codex pets directory.

---

## File And Artifact Map

- Read: `docs/superpowers/specs/2026-05-24-zircon-neko-pet-design.md`
- Create: `C:/Users/HeJiahui/.codex/tmp/hatch-pet/zircon-neko/pet_request.json`
- Create: `C:/Users/HeJiahui/.codex/tmp/hatch-pet/zircon-neko/imagegen-jobs.json`
- Create: `C:/Users/HeJiahui/.codex/tmp/hatch-pet/zircon-neko/prompts/*.md`
- Create during generation: `C:/Users/HeJiahui/.codex/tmp/hatch-pet/zircon-neko/decoded/*.png`
- Create during processing: `C:/Users/HeJiahui/.codex/tmp/hatch-pet/zircon-neko/final/spritesheet.webp`
- Create during processing: `C:/Users/HeJiahui/.codex/tmp/hatch-pet/zircon-neko/final/validation.json`
- Create during QA: `C:/Users/HeJiahui/.codex/tmp/hatch-pet/zircon-neko/qa/contact-sheet.png`
- Create during QA: `C:/Users/HeJiahui/.codex/tmp/hatch-pet/zircon-neko/qa/previews/*.gif`
- Create during QA: `C:/Users/HeJiahui/.codex/tmp/hatch-pet/zircon-neko/qa/review.json`
- Create package: `C:/Users/HeJiahui/.codex/pets/zircon-neko/pet.json`
- Create package: `C:/Users/HeJiahui/.codex/pets/zircon-neko/spritesheet.webp`

No ZirconEngine Rust, Slint, Cargo, editor, runtime, hub, or plugin source files are modified.

## Milestone 1: Prepare The New Hatch Run

- Goal: Produce a fresh hatch-pet run directory whose manifest and prompts encode the approved Zircon Neko concept.
- In-scope behaviors: pet name, description, chibi anime style, visual identity notes, reference-style avoidances, output directory, base job, all nine animation row jobs, layout guides, and chroma-key configuration.
- Dependencies: approved `Zircon Neko` design spec.
- Implementation slices:
  - [ ] Read `docs/superpowers/specs/2026-05-24-zircon-neko-pet-design.md` and extract the approved name, visual identity, style, animation states, and avoidances.
  - [ ] Run `C:/Users/HeJiahui/.codex/skills/hatch-pet/scripts/prepare_pet_run.py` with:

    ```powershell
    python "C:/Users/HeJiahui/.codex/skills/hatch-pet/scripts/prepare_pet_run.py" `
      --pet-name "Zircon Neko" `
      --description "A focused Japanese anime chibi blonde catgirl architect mascot in a sailor-uniform-led outfit with a small zircon-blue engineer accessory." `
      --output-dir "C:/Users/HeJiahui/.codex/tmp/hatch-pet/zircon-neko" `
      --pet-notes "Non-sexual Japanese anime chibi blonde catgirl architect mascot; light blonde hair, visible cat ears, fluffy blonde tail, blue-purple eyes, cheerful but focused expression, sailor-uniform-led outfit with compact white sailor blouse, blue sailor collar, blue neck ribbon or tie, pleated blue-and-white skirt, gold trim, small dark-gray work accents, small blue-cyan zircon crystal brooch or badge; no copied reference hat, staff, companion creature, exact costume silhouette, text, or logos." `
      --style-preset "sticker" `
      --style-notes "Chibi anime sticker rendering, 2.5-head proportions, thick clean outline, bright eyes, soft blush, white/blue/gold sailor-uniform palette, readable inside 192x208 cells, family-friendly and non-sexual." `
      --force
    ```

  - [ ] Inspect `imagegen-jobs.json` and confirm it contains `base`, `idle`, `running-right`, `running-left`, `waving`, `jumping`, `failed`, `waiting`, `running`, and `review`.
  - [ ] Confirm each row job lists its layout guide and depends on the canonical base as required by the hatch-pet workflow.
- Testing stage:
  - [ ] Verify these files exist: `pet_request.json`, `imagegen-jobs.json`, `prompts/base-pet.md`, and `references/layout-guides/idle.png`.
  - [ ] Check the manifest has exactly ten jobs: one base job and nine row jobs.
  - [ ] Check prompts prohibit readable text, logos, copied reference-specific items, sexualized styling, detached effects, shadows, speed lines, wave marks, and scene backgrounds.
- Lightweight checks:
  - [ ] Use PowerShell JSON parsing to inspect manifest shape instead of editing it manually.
- Exit evidence:
  - [ ] `imagegen-jobs.json` has all expected jobs and no completed jobs before visual generation starts.
  - [ ] `pet_request.json` records the display name `Zircon Neko`.

## Milestone 2: Generate The Canonical Anime Catgirl Base

- Goal: Create the canonical full-body Zircon Neko sailor-uniform image that locks the final identity.
- In-scope behaviors: one centered chibi catgirl, flat chroma background, readable full-body silhouette, cat ears, tail, blue-purple eyes, white/blue/gold sailor uniform, zircon-blue accessory, no copied reference-specific props, no sexualized styling, no text, no scenery, no shadows, no detached effects.
- Dependencies: Milestone 1 run directory and `prompts/base-pet.md`.
- Implementation slices:
  - [ ] Use the image generation skill with the base prompt from `C:/Users/HeJiahui/.codex/tmp/hatch-pet/zircon-neko/prompts/base-pet.md`.
  - [ ] Use the user-provided image only as broad style reference for chibi proportions, polished anime sticker rendering, bright eyes, and white/blue/gold palette.
  - [ ] Select the strongest generated source where the face, hair, cat ears, tail, sailor collar, neck ribbon/tie, pleated skirt, zircon brooch, and compact silhouette match the approved design.
  - [ ] Copy the selected source into the base job's decoded `output_path`.
  - [ ] Copy the decoded base to `C:/Users/HeJiahui/.codex/tmp/hatch-pet/zircon-neko/references/canonical-base.png`.
  - [ ] Mark the `base` job complete in `imagegen-jobs.json` with the selected source path and completion timestamp.
- Testing stage:
  - [ ] Verify `references/canonical-base.png` exists.
  - [ ] Visually check the base image for one centered full-body pet, clear chroma background, primary sailor-uniform read, no copied reference hat/staff/companion, no sexualized styling, no text, no shadows, no scenery, and no disconnected decorative pieces.
  - [ ] If the base fails visual identity, regenerate only the base before producing any row strips.
- Lightweight checks:
  - [ ] Confirm row jobs remain incomplete after marking only `base` complete.
- Exit evidence:
  - [ ] `base` job has `status: "complete"`.
  - [ ] `references/canonical-base.png` is the same selected image used for decoded base output.

## Milestone 3: Generate Animation Rows

- Goal: Generate all nine animation row strips while preserving the canonical Zircon Neko identity.
- In-scope behaviors: row-specific motion semantics, exact frame count per row, layout-guide conformance, chroma-key background, no guide marks, no forbidden effects, and consistent catgirl identity.
- Dependencies: Milestone 2 canonical base.
- Implementation slices:
  - [ ] Generate `idle` from its row prompt with canonical base and layout guide inputs.
  - [ ] Generate `running-right` from its row prompt with canonical base and layout guide inputs.
  - [ ] Inspect `running-right`; mirror it into `running-left` only if the cat ears, tail, brooch side, hair shape, and outfit asymmetry remain acceptable when flipped, otherwise generate `running-left` normally.
  - [ ] Generate `waving`, `jumping`, `failed`, `waiting`, `running`, and `review` from their row prompts with canonical base and matching layout guide inputs.
  - [ ] For each accepted generated row, copy the selected source into that job's decoded output path and mark the job complete in `imagegen-jobs.json`.
  - [ ] If image generation returns a transport-level bad request for a row, retry that same row once with its retry prompt and the same listed inputs.
- Testing stage:
  - [ ] Confirm every row job has `status: "complete"`.
  - [ ] Inspect each accepted strip for exact row semantics:
    - `idle`: blink, tiny body bob, and tail sway.
    - `running-right`: light rightward chibi steps with tail balance.
    - `running-left`: light leftward chibi steps with tail balance.
    - `waving`: small friendly hand wave, no wave marks.
    - `jumping`: catlike hop with ears lifting, no floor effects.
    - `failed`: ears droop, tail lowers, disappointed expression, no floating symbols.
    - `waiting`: expectant confirmation pose.
    - `running`: focused task-processing, not literal sprinting.
    - `review`: attentive lean-in inspection without new props.
  - [ ] Regenerate only rows that fail identity, layout, clipping, background, effect, copied-reference, sexualization, or semantic checks.
- Lightweight checks:
  - [ ] Prefer manifest inspection over opening all intermediate images in the parent thread.
- Exit evidence:
  - [ ] All visual jobs are complete and all decoded row outputs exist.

## Milestone 4: Compose, Validate, And QA

- Goal: Convert accepted generated strips into a validated Codex pet atlas and visual QA artifacts.
- In-scope behaviors: frame extraction, component inspection, atlas composition, WebP output, atlas validation, contact sheet, motion previews, row-by-row visual review, and repair loop.
- Dependencies: Milestone 3 completed decoded outputs.
- Implementation slices:
  - [ ] Create final and QA directories:

    ```powershell
    New-Item -ItemType Directory -Force "C:/Users/HeJiahui/.codex/tmp/hatch-pet/zircon-neko/final", "C:/Users/HeJiahui/.codex/tmp/hatch-pet/zircon-neko/qa"
    ```

  - [ ] Run frame extraction:

    ```powershell
    python "C:/Users/HeJiahui/.codex/skills/hatch-pet/scripts/extract_strip_frames.py" `
      --decoded-dir "C:/Users/HeJiahui/.codex/tmp/hatch-pet/zircon-neko/decoded" `
      --output-dir "C:/Users/HeJiahui/.codex/tmp/hatch-pet/zircon-neko/frames" `
      --states all `
      --method auto
    ```

  - [ ] Run frame inspection:

    ```powershell
    python "C:/Users/HeJiahui/.codex/skills/hatch-pet/scripts/inspect_frames.py" `
      --frames-root "C:/Users/HeJiahui/.codex/tmp/hatch-pet/zircon-neko/frames" `
      --json-out "C:/Users/HeJiahui/.codex/tmp/hatch-pet/zircon-neko/qa/review.json" `
      --require-components
    ```

  - [ ] Compose the atlas:

    ```powershell
    python "C:/Users/HeJiahui/.codex/skills/hatch-pet/scripts/compose_atlas.py" `
      --frames-root "C:/Users/HeJiahui/.codex/tmp/hatch-pet/zircon-neko/frames" `
      --output "C:/Users/HeJiahui/.codex/tmp/hatch-pet/zircon-neko/final/spritesheet.png" `
      --webp-output "C:/Users/HeJiahui/.codex/tmp/hatch-pet/zircon-neko/final/spritesheet.webp"
    ```

  - [ ] Validate the atlas:

    ```powershell
    python "C:/Users/HeJiahui/.codex/skills/hatch-pet/scripts/validate_atlas.py" `
      "C:/Users/HeJiahui/.codex/tmp/hatch-pet/zircon-neko/final/spritesheet.webp" `
      --json-out "C:/Users/HeJiahui/.codex/tmp/hatch-pet/zircon-neko/final/validation.json"
    ```

  - [ ] Generate the contact sheet:

    ```powershell
    python "C:/Users/HeJiahui/.codex/skills/hatch-pet/scripts/make_contact_sheet.py" `
      "C:/Users/HeJiahui/.codex/tmp/hatch-pet/zircon-neko/final/spritesheet.webp" `
      --output "C:/Users/HeJiahui/.codex/tmp/hatch-pet/zircon-neko/qa/contact-sheet.png"
    ```

  - [ ] Generate motion previews:

    ```powershell
    python "C:/Users/HeJiahui/.codex/skills/hatch-pet/scripts/render_animation_previews.py" `
      --frames-root "C:/Users/HeJiahui/.codex/tmp/hatch-pet/zircon-neko/frames" `
      --output-dir "C:/Users/HeJiahui/.codex/tmp/hatch-pet/zircon-neko/qa/previews"
    ```

  - [ ] Visually inspect the contact sheet and previews. If row scale pops due to extraction but the source strip is stable, rerun extraction with `--method stable-slots`, rerun inspection with `--allow-stable-slots`, then rerun atlas composition, validation, contact sheet, and previews.
  - [ ] If a row is semantically wrong or visually inconsistent, regenerate only that row and rerun this milestone.
- Testing stage:
  - [ ] `qa/review.json` has no errors.
  - [ ] `final/validation.json` reports a valid 1536x1872 atlas with transparent unused cells.
  - [ ] `qa/contact-sheet.png` shows the same Zircon Neko identity in all nine rows.
  - [ ] `qa/previews/*.gif` show state-matching movement without unintended size popping, wrong facing direction, or stagnant loops.
- Lightweight checks:
  - [ ] Use the generated JSON reports to identify deterministic failures before visual repair.
- Exit evidence:
  - [ ] `final/spritesheet.webp`, `final/validation.json`, `qa/review.json`, `qa/contact-sheet.png`, and row preview GIFs all exist and pass review.

## Milestone 5: Package The Pet

- Goal: Install the final validated pet in the local Codex pets directory.
- In-scope behaviors: package directory creation, final WebP copy, `pet.json` creation, run summary, and retained QA artifacts.
- Dependencies: Milestone 4 validated atlas.
- Implementation slices:
  - [ ] Create the package directory:

    ```powershell
    New-Item -ItemType Directory -Force "C:/Users/HeJiahui/.codex/pets/zircon-neko"
    ```

  - [ ] Copy the final atlas:

    ```powershell
    Copy-Item -Force `
      "C:/Users/HeJiahui/.codex/tmp/hatch-pet/zircon-neko/final/spritesheet.webp" `
      "C:/Users/HeJiahui/.codex/pets/zircon-neko/spritesheet.webp"
    ```

  - [ ] Write `pet.json` with:

    ```json
    {
      "id": "zircon-neko",
      "displayName": "Zircon Neko",
      "description": "A focused Japanese anime chibi blonde catgirl architect mascot in a sailor-uniform-led outfit with a small zircon-blue engineer accessory.",
      "spritesheetPath": "spritesheet.webp"
    }
    ```

  - [ ] Write `qa/run-summary.json` containing the run directory, final spritesheet, validation JSON, contact sheet, review JSON, and package directory.
  - [ ] Remove intermediate prompt files, layout guides, decoded strips, extracted frames, PNG atlas, and imagegen manifest only after the package and QA artifacts pass.
- Testing stage:
  - [ ] Confirm `C:/Users/HeJiahui/.codex/pets/zircon-neko/pet.json` exists.
  - [ ] Confirm `C:/Users/HeJiahui/.codex/pets/zircon-neko/spritesheet.webp` exists.
  - [ ] Confirm `pet.json` references `spritesheet.webp`.
  - [ ] Confirm `qa/run-summary.json` records `ok: true`.
- Lightweight checks:
  - [ ] Parse `pet.json` as JSON before declaring completion.
- Exit evidence:
  - [ ] The package directory contains exactly the installable pet manifest and spritesheet.
  - [ ] The run directory retains final validation, contact sheet, previews, review report, and run summary.

## Plan Self-Review

- Spec coverage: The approved name, catgirl identity, reference-handling rules, non-sexual mascot requirement, visual identity, nine animation rows, generation method, QA rules, and out-of-scope source-code restriction are covered by Milestones 1-5.
- Placeholder scan: No placeholder markers, vague test instructions, or incomplete task references remain.
- Type and path consistency: The plan consistently uses `zircon-neko` for package/run IDs, `Zircon Neko` for display name, and the local Codex paths under `C:/Users/HeJiahui/.codex`.
