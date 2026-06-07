import { grid } from "../../foundation/layout.js";
import { panelGroup } from "../shared/module-components.js";
import { atomPalette } from "./center/atom-palette.js";
import { collectionPalette } from "./center/collection-palette.js";
import { componentCoverageMatrix } from "./center/coverage-matrix.js";
import { layoutGrammarPanel } from "./center/layout-grammar.js";
import { surfacePalette } from "./center/surface-palette.js";

export function componentLabCenter() {
  return grid({ className: "zr-module-editor-grid is-component-lab", children: [
    panelGroup("component-lab-main", [
      { label: "Atoms", active: true, content: atomPalette() },
      { label: "Collections", content: collectionPalette() },
      { label: "Surfaces", content: surfacePalette() }
    ], { className: "is-module-center" }),
    componentCoverageMatrix(),
    layoutGrammarPanel()
  ] });
}
