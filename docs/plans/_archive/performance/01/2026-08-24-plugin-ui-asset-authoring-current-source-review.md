---
title: Plugin UI Asset Authoring Current Source Review
date: 2026-08-24
scope:
  - zircon_plugins/ui_asset_authoring
status: static_complete_product_and_dynamic_pending
canonical_owners:
  - docs/plans/optimize/zircon_plugins/08-first-party-editor-authoring-extension-document-operation-toolkit-runtime-contract-product-integration-review.md
  - docs/plans/optimize/zircon_editor/23-ui-asset-hud-widget-binding-theme-icon-accessibility-menu-flow-font-atlas-authoring-review.md
references:
  - dev/UnrealEngine/Engine/Source/Editor/UMGEditor/Private/WidgetBlueprintEditor.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UMGEditor/Private/BlueprintModes/WidgetDesignerApplicationMode.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UMGEditor/Private/Designer/SDesignerView.cpp
---

# Plugin UI Asset Authoring Current Source Review

## 1. Coverage

The current Rust surface is **6/6 files**, **459 physical / 426 non-empty lines**, **17,968 bytes**, and **4 test markers**. Its workspace-relative `path + LF + decoded text + LF` SHA-256 is `a1a4267117796849c623b8666d147b9c717c261722d4a404e368df71c2576151`. The generated manifest, both Cargo manifests, physical package inventory, first-party Editor catalog, core UI asset editor identity/session chain and Plugins08/Editor23 owner reports were also checked. The plugin directory is clean.

## 2. Primary finding

There is no authoring algorithm in this package to optimize. The source plugin registers descriptors for one view/drawer/template, three asset-type augmentations and three create commands. Actual document/session, designer, preview, inspector, binding and retained rendering behavior lives in `zircon_editor` under the same `editor.ui_asset` identity.

The package contains **zero physical UI/template assets**, yet registration references four `plugins://ui_asset_authoring/...` documents: `editor/authoring.zui` plus layout/widget/style templates. All four resource edges are unresolved. The three create commands dispatch `OpenView(editor.ui_asset)`; they do not create a document, allocate an asset identity, write a file, enter a transaction or return a creation receipt.

Core also enables `editor.extension.ui_asset_authoring` by default through `EditorSubsystemReport` and registers its builtin UI asset editor using the same view ID. A real source mount can collide with the builtin authority. The first-party Editor catalog does not link this package, while the dist has no command/event/bridge/host-ready/state/unload behavior and no serialized extensions. Source is unreachable by default; dist cannot recreate source contributions.

Optimizing the bounded descriptor vectors would reduce one-time registration work while leaving creation, resources, product reachability and document ownership broken. No production edit is appropriate before the owner hard cut.

## 3. Unreal source constraints

Unreal's UMG editor keeps a coherent toolkit owner. `FWidgetBlueprintEditor` owns the edited blueprint, command bindings, compile/debug integration and preview notifications. `FWidgetDesignerApplicationMode` registers designer/compiler/hierarchy/palette/detail tab factories and a versioned layout around the same editor instance. `SDesignerView` subscribes to selection and preview-recreation signals, owns preview/hit-test state and responds to explicit invalidation rather than acting as a second document owner.

The transferable boundary is one document/toolkit authority with real resource identity, compile/preview generations, tabs as projections and extension hooks that decorate that authority. Zircon should not duplicate Unreal's Blueprint object model, but it must stop treating a view descriptor and missing template URI as an asset editor product.

## 4. Dependency-ordered plan

### M0: fail closed and choose the authority

Add product tests that mount this source registration into the default Editor and prove the duplicate view, four missing resources, fake create behavior and catalog omission. Choose Editor23's core document/toolkit as the unique authority, with this package either decorating it through explicit slots or replacing it atomically. Remove the losing view/open-command identity.

### M1: real resource and create contracts

Every retained package URI must resolve to a manifest-declared, hashed, schema-valid resource owned by the package version. Create Layout/Widget/Style must allocate a typed document/asset identity, stage default content, commit through the document transaction, mark dirty/save state correctly and open the resulting toolkit. Failure produces zero file, registry and UI side effects.

### M2: source/dist behavior equivalence

Generate one contribution/resource bundle for source and dist. Native materialization must reconstruct templates, asset-type augmentation and an executable create/open bridge, or the dist must withdraw readiness. Mount/unmount uses provider generation, revokes commands/resources/toolkits and waits for active document leases.

### M3: editor performance architecture

Keep compilation, preview and render state under Editor23. Publish independent document, compiled UI, selection, style/theme, layout and preview generations. A stable frame performs no parse, compile, full hierarchy rebuild or template rematerialization. Edits invalidate only affected dependency/style/layout/paint domains; background compilation publishes last-good artifacts and structured diagnostics.

### M4: product and dynamic qualification

Exercise create/open/edit/undo/redo/save/reopen/rename/delete, invalid source, multi-document, source/dist enable/disable and dependency/style/font changes. WPR/ETW records editor/main/render thread CPU, parse/compile queue, invalidation, allocations, RSS and energy. RenderDoc verifies current-source preview pixels, draw calls, scissor/clip, text/font atlas and resource lifetime.

## 5. Acceptance

1. All four package URIs resolve and pass owner/hash/schema validation before capability publication; missing resources fail the entire mount atomically.
2. Exactly one `editor.ui_asset` document/toolkit/view authority exists. Plugin contribution declares replace/decorate/extend explicitly and cannot collide silently.
3. Each create action produces one typed asset/document transaction and opens that identity; cancel/failure leaves zero files, registry records and dirty documents.
4. Source and dist expose equivalent contributions, resources and executable behavior. Package disable revokes new entry points while active documents follow an explicit close/save/cancel lease.
5. Stable editor refresh has zero parse/compile/template/layout rebuild work. One node/property/style change visits only the affected dependency and retained projection closure.
6. Measure `1/100/1,000/10,000` nodes, `1/16/64` open documents and small/large style/font sets. Record input-to-preview p50/p95/p99, compile and queue time, changed nodes, layout/paint visits, draw calls, allocations, RSS/GPU memory and energy.
7. Compare with Unreal only on matched widget trees, preview resolution, font/theme state, interaction and hardware; architecture evidence is not performance parity.

## 6. Validation status

- Static per-Rust-file review: **6/6 complete**.
- Physical package resources: **0 present / 4 referenced**, product gate failed statically.
- Global plugin structure audit: **pass** for manifest/schema/registration/dist boundaries; it does not validate resource closure or behavior.
- `rustfmt --check`: **pass** for all 6 Rust files.
- Cargo, default-host mount, create/save/reopen, source/dist equivalence, WPR/ETW, RenderDoc and power validation: **pending**.
- This module is not eligible for protected-ledger acceptance, milestone commit or WeCom completion notification.
