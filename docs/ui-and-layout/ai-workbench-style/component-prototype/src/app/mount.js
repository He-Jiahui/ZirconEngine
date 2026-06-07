import { createWorkbenchController } from "./controller.js";
import { bindClickInteractions } from "./interactions/click.js";
import { bindFieldInteractions } from "./interactions/fields.js";
import { bindHistoryInteractions } from "./interactions/history.js";
import { bindKeyboardActivation } from "./interactions/keyboard.js";

export function mountWorkbenchApp(app) {
  const controller = createWorkbenchController(app);
  controller.renderWorkbench();
  controller.activateLocationModuleState({ silent: true });
  bindClickInteractions(controller);
  bindFieldInteractions(controller);
  bindKeyboardActivation();
  bindHistoryInteractions(controller);
  return controller;
}
