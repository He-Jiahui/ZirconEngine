import { useEffect, useState } from "react";
import { HubWindow } from "./components/shell";
import { fallbackShellState } from "./data/hubData";
import { loadHubState } from "./tauri/hubApi";
import type { HubShellState } from "./types/hub";

export function App() {
  const [state, setState] = useState<HubShellState>(fallbackShellState);

  useEffect(() => {
    let cancelled = false;
    loadHubState().then((nextState) => {
      if (!cancelled) {
        setState(nextState);
      }
    });
    return () => {
      cancelled = true;
    };
  }, []);

  return <HubWindow state={state} />;
}
