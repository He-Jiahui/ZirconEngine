export function actionClickTarget(event) {
  return event.target.closest("[data-action]");
}
