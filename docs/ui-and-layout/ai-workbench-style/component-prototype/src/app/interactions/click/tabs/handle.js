import { handledClick, ignoredClick } from "../utils.js";
import { recordPlainTabFeedback, setTabStatus } from "./feedback.js";
import { applyPanelTabRoute } from "./panel.js";
import { activateTabState } from "./state.js";
import { tabClickTarget } from "./target.js";

export function handleTabClick(event, controller) {
  const tab = tabClickTarget(event);
  if (!tab) return ignoredClick;

  activateTabState(tab);
  if (!applyPanelTabRoute(tab, controller)) {
    recordPlainTabFeedback(controller, tab);
  }
  setTabStatus(controller, tab);
  return handledClick;
}
