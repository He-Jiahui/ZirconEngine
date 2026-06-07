import { dispatchClickInteraction } from "./dispatch.js";
import { clickHandlers } from "./handlers.js";

export function bindClickInteractions(controller) {
  document.addEventListener("click", (event) => {
    dispatchClickInteraction(event, controller, clickHandlers);
  });
}
