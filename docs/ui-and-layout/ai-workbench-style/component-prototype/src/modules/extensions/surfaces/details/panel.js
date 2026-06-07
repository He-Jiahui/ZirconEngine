import { panelGroup } from "../../../shared/module-components.js";
import { extensionDetailStatusPanel } from "./status.js";
import { extensionDetailTablePanel } from "./table.js";
import { extensionSummaryPanel } from "./summary.js";

export function extensionDetails(config) {
  const [firstTab, secondTab, thirdTab] = config.detailTabs;
  return panelGroup(`${config.id}-right`, [
    { label: firstTab, active: true, content: extensionSummaryPanel(config) },
    { label: secondTab, content: extensionDetailTablePanel(config, secondTab) },
    { label: thirdTab, content: extensionDetailStatusPanel(config) }
  ], { className: "is-extension-right" });
}
