export function tabClickTarget(event) {
  return event.target.closest(".zr-tab, .zr-segment-item, .zr-panel-tab");
}
