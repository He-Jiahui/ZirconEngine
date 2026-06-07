import { historyInteractionEvents } from "./events.js";

export function bindHistoryInteractions(controller) {
  historyInteractionEvents.forEach((eventName) => {
    window.addEventListener(eventName, controller.activateLocationModuleState);
  });
}
