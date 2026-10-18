import unittest
from pathlib import Path


class HubA5StateChronologyContractTests(unittest.TestCase):
    """Static guard for Hub's epoch/revision ordering boundary."""

    def setUp(self) -> None:
        self.repo_root = Path(__file__).resolve().parents[2]
        web = self.repo_root / "zircon_hub/web/src"
        self.chronology = web / "tauri/hubStateChronology.ts"
        self.validator = web / "tauri/hubStateValidator.ts"
        self.api = web / "tauri/hubApi.ts"
        self.app = web / "App.tsx"
        self.regressions = self.repo_root / "zircon_hub/web/tests/hub_ui_regressions.test.mjs"

    def test_chronology_compares_lossless_revision_and_rejects_stale_lanes(self) -> None:
        source = self.chronology.read_text(encoding="utf-8")
        for anchor in (
            "const MAX_STAGED_EVENT_EPOCHS = 4;",
            "private acceptedEpoch: string | undefined;",
            "private acceptedRevision = -1n;",
            "function revision(state: HubShellState): bigint",
            "BigInt(state.stateRevision)",
            "private readonly stagedEvents = new Map<string, HubShellState>();",
            "if (state.backendEpoch !== this.acceptedEpoch || revision(state) <= this.acceptedRevision)",
            "this.acceptedRevision = revision(state);",
            "if (stagedState && revision(stagedState) >= revision(state))",
            "this.stagedEvents.size >= MAX_STAGED_EVENT_EPOCHS",
        ):
            self.assertIn(anchor, source)
        accept = source.split("accept(state:", 1)[1].split("private stageEvent", 1)[0]
        self.assertLess(accept.index("state.backendEpoch !== this.acceptedEpoch"), accept.index("this.acceptedRevision = revision(state)"))
        self.assertIn("return undefined;", accept)

    def test_bootstrap_is_a_transaction_and_foreign_epochs_remain_quarantined(self) -> None:
        source = self.chronology.read_text(encoding="utf-8")
        for anchor in (
            "beginBootstrap()",
            "this.bootstrapInFlight = true;",
            "acceptBootstrap(state: HubShellState)",
            "const stagedState = this.stagedEvents.get(state.backendEpoch);",
            "const hasForeignEpoch = [...this.stagedEvents.keys()].some((epoch) => epoch !== state.backendEpoch);",
            "this.stagedEvents.delete(state.backendEpoch);",
            "this.stagedEvents.clear();",
            "const candidate = stagedState && revision(stagedState) > revision(state) ? stagedState : state;",
            "quarantine()",
            "this.quarantined = true;",
            "if (this.quarantined)",
            "export const MAX_AUTOMATIC_BOOTSTRAP_RETRIES = 2;",
        ):
            self.assertIn(anchor, source)
        bootstrap = source.split("acceptBootstrap(state:", 1)[1].split("accept(state:", 1)[0]
        self.assertLess(bootstrap.index("const hasForeignEpoch"), bootstrap.index("const candidate"))
        self.assertLess(bootstrap.index("const candidate"), bootstrap.index("this.acceptedEpoch = candidate.backendEpoch"))

    def test_validator_and_api_preserve_epoch_revision_wire_shape(self) -> None:
        validator = self.validator.read_text(encoding="utf-8")
        api = self.api.read_text(encoding="utf-8")
        for anchor in (
            '"backendEpoch"',
            '"stateRevision"',
            "state.stateRevision as string",
            "canonical decimal string",
            "backendEpoch as string).trim()",
            "fallbackStateRevision = 0n",
            "fallbackStateRevision += 1n",
            "fallbackStateRevision.toString()",
            "assertHubShellState(payload)",
            "assertHubShellState(\n    await invoke<unknown>(\"hub_action\"",
            "assertHubShellState(event.payload)",
        ):
            self.assertIn(anchor, validator + api)
        self.assertNotIn("Number(state.stateRevision", validator)
        self.assertNotIn("parseInt(state.stateRevision", validator)

    def test_app_routes_bootstrap_invoke_and_event_states_through_one_acceptance_gate(self) -> None:
        source = self.app.read_text(encoding="utf-8")
        for anchor in (
            "const chronologyRef = useRef(",
            "function applyHubState(nextState: HubShellState, source: HubStateSource): boolean",
            "const acceptedState = acceptHubState(chronologyRef.current, nextState, source);",
            "applyHubState(outcome.state, \"bootstrap\")",
            "applyHubState(nextState, \"invoke\");",
            "applyHubState(nextState, \"event\");",
            "subscribeHubStateChanged(",
            "chronologyRef.current.quarantine();",
            "bootstrapSequenceRef.current += 1;",
        ):
            self.assertIn(anchor, source)
        gate = source.split("function applyHubState", 1)[1].split("const reloadHubState", 1)[0]
        self.assertIn("acceptHubState(chronologyRef.current, nextState, source)", gate)
        self.assertIn("if (!acceptedState)", gate)
        self.assertLess(source.index('applyHubState(outcome.state, "bootstrap")'), source.index('applyHubState(nextState, "invoke")'))

    def test_regressions_cover_stale_duplicate_foreign_epoch_retry_and_bigint_edges(self) -> None:
        source = self.regressions.read_text(encoding="utf-8")
        for test_name in (
            'test("Hub chronology rejects delayed duplicate and retired-epoch states"',
            'test("Hub chronology requires bootstrap before events or invoke responses"',
            'test("Hub chronology retains a new-epoch event and rejects an old bootstrap response"',
            'test("Hub chronology preserves the foreign epoch when A and B are staged before stale A bootstrap"',
            'test("Hub chronology quarantines retired-epoch events while foreign bootstrap evidence is pending"',
            'test("Hub chronology exposes a retry requirement after a stale bootstrap"',
            'test("Hub bootstrap retry budget is bounded and resettable"',
            'test("Hub state parser requires lossless chronology strings"',
        ):
            self.assertIn(test_name, source)
        self.assertIn("18446744073709551617", source)


if __name__ == "__main__":
    unittest.main()
