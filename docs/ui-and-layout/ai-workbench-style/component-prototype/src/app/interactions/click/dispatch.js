export function dispatchClickInteraction(event, controller, clickHandlers) {
  let handled = false;
  for (const handleClick of clickHandlers) {
    const result = handleClick(event, controller, handled);
    handled = handled || result.handled;
    if (result.stop) return;
  }
}
