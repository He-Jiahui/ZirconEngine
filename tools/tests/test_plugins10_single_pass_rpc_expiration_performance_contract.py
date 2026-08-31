import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
DISPATCH_SOURCE = (
    REPO_ROOT
    / "zircon_plugins"
    / "net"
    / "features"
    / "rpc"
    / "runtime"
    / "src"
    / "manager"
    / "dispatch.rs"
)


def function_body(source: str, signature: str) -> str:
    start = source.index(signature)
    opening = source.index("{", start)
    depth = 0
    for index in range(opening, len(source)):
        character = source[index]
        if character == "{":
            depth += 1
        elif character == "}":
            depth -= 1
            if depth == 0:
                return source[opening + 1 : index]
    raise AssertionError(f"unterminated function body for {signature}")


class Plugins10SinglePassRpcExpirationPerformanceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        source = DISPATCH_SOURCE.read_text(encoding="utf-8")
        cls.body = function_body(
            source,
            "pub fn expire_pending_requests(&self) -> Vec<RpcDispatchReport>",
        )

    def test_expiration_sweeps_pending_requests_with_retain(self) -> None:
        self.assertIn("state.pending_requests.retain", self.body)

    def test_expiration_does_not_materialize_request_ids(self) -> None:
        self.assertNotIn(".collect::<Vec<_>>()", self.body)
        self.assertNotIn("then_some(*request)", self.body)

    def test_expiration_does_not_rehash_ids_in_a_second_pass(self) -> None:
        self.assertNotIn("expired.into_iter()", self.body)
        self.assertNotIn("state.pending_requests.remove(&request)", self.body)

    def test_expiration_builds_reports_inside_the_removal_branch(self) -> None:
        self.assertIn("expired_reports.push", self.body)
        self.assertIn("RpcDispatchStatus::TimedOut", self.body)
        self.assertIn("pending RPC request timed out", self.body)


if __name__ == "__main__":
    unittest.main()
