import { moduleTable, panel, tag } from "../../shared/module-components.js";
import { componentCoverage } from "../data.js";
import { componentLabRouteOptions } from "../routes.js";

export function componentCoverageMatrix() {
  return panel("Coverage Matrix", moduleTable(
    ["Family", "Functional Path", "Coverage", "Layer"],
    componentCoverage.map(([family, path, coverage, layer], index) => ({
      cells: [family, path, coverage, tag(layer, index < 6 ? "cyan" : "blue")],
      selected: index === 0
    })),
    "0.8fr 1.45fr 1.35fr 0.7fr",
    componentLabRouteOptions("component-lab-right:inputs", "workbench.component_lab.coverage")
  ));
}
