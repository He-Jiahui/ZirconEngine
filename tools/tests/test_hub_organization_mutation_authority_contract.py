"""Static contract for Hub organization, member, and invitation authority."""

from __future__ import annotations

import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
HUB = ROOT / "zircon_hub"
ORGANIZATION = HUB / "src/service/organization/mod.rs"
INVITATION = HUB / "src/service/organization/invitation.rs"
QUERY = HUB / "src/service/organization/query.rs"
TESTS = HUB / "src/service/organization/tests/cases.rs"
AUTHORITY_TESTS = HUB / "src/service/organization/tests/invitation_authority_tests.rs"


def production(source: str) -> str:
    return source.split("#[cfg(test)]", 1)[0]


class HubOrganizationMutationAuthorityContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.organization = production(ORGANIZATION.read_text(encoding="utf-8"))
        cls.invitation = production(INVITATION.read_text(encoding="utf-8"))
        cls.query = production(QUERY.read_text(encoding="utf-8"))
        cls.tests = TESTS.read_text(encoding="utf-8")
        cls.authority_tests = AUTHORITY_TESTS.read_text(encoding="utf-8")

    def test_mutations_are_typed_and_share_immediate_receipt_transactions(self) -> None:
        for action in (
            "CreateProject",
            "SetMember",
            "TransferOwnership",
            "Invite",
            "AcceptInvite",
            "RevokeInvite",
        ):
            self.assertIn(action, self.organization, action)
        for anchor in (
            "pub fn create(",
            "pub fn mutate(",
            "TransactionBehavior::Immediate",
            "receipt::fingerprint(",
            "receipt::replay(&transaction, principal",
            "receipt::commit(",
            "transaction.commit()?;",
            "expected_policy_revision",
            "UPDATE organizations SET policy_revision=?2",
            "audit(",
        ):
            self.assertIn(anchor, self.organization, anchor)
        self.assertIn("expected <= 0 || expected.to_string() != request.expected_policy_revision", self.organization)
        self.assertIn("next_revision = revision.checked_add(1)", self.organization)

    def test_role_matrix_and_owner_transfer_are_fail_closed(self) -> None:
        for anchor in (
            'Some("owner" | "admin" | "member")',
            'actor_role.as_deref() == Some("owner")',
            'matches!(actor_role.as_deref(), Some("owner" | "admin"))',
            "if !allowed",
            'existing.as_deref() == Some("owner") || existing.is_none()',
            "Ownership changes use TransferOwnership",
            "issuer == &principal.issuer && subject == &principal.subject",
            "UPDATE memberships SET role='owner'",
            "UPDATE memberships SET role='admin'",
            "changed != 1",
            "validate_identity(issuer, subject)?",
            "validate_role(role)?",
        ):
            self.assertIn(anchor, self.organization, anchor)

    def test_invitation_acceptance_rechecks_qualified_authority_inside_write(self) -> None:
        for anchor in (
            "pub(super) fn authorized_role(",
            "JOIN memberships sender",
            "sender.active=1",
            "sender.role IN ('owner','admin')",
            "recipient.active=1",
            "i.status='pending'",
            "i.expires_at>?5",
            "Permanent status prevents old invitations",
            "UPDATE invitations SET status='revoked'",
            "revoke_issued",
        ):
            self.assertIn(anchor, self.invitation, anchor)
        self.assertIn("invitation_role = if let Mutation::AcceptInvite", self.organization)
        self.assertIn("replayed.is_some()", self.organization)

    def test_queries_are_tenant_scoped_and_cursor_bounded(self) -> None:
        self.assertIn("if !(1..=100).contains(&limit)", self.query)
        self.assertIn("receipt::validate_id(after)?", self.query)
        for anchor in (
            "WHERE organization_id=?1",
            "m.issuer=?1 AND m.subject=?2 AND m.active=1",
            "LIMIT ?3",
            "LIMIT ?4",
            "role IN ('owner','admin')",
            "CASE WHEN i.status='pending' AND i.expires_at<=?3 THEN 'expired' ELSE i.status END",
        ):
            self.assertIn(anchor, self.query, anchor)
        self.assertIn("let next_cursor = more.then", self.query)

    def test_rust_regressions_cover_tenant_conflict_replay_and_owner_edges(self) -> None:
        for name in (
            "tenant_isolation_stale_revision_and_revocation_are_transactional",
            "failed_audit_rolls_back_the_entire_mutation",
            "cancelled_response_can_be_reconciled_and_retried_without_a_second_commit",
            "invitee_can_discover_revision_accept_and_replay_a_receipt",
            "issued_invitation_listing_is_paginated_tenant_scoped_and_freshly_authorized",
            "old_and_new_invitations_cannot_override_owner_membership_changes",
            "ownership_transfer_is_atomic_and_only_current_owner_can_start_it",
        ):
            self.assertIn(name, self.tests, name)
        for name in (
            "inviter_revocation_is_permanent_for_pending_invites_and_audited",
            "acceptance_checks_fresh_qualified_inviter_membership_in_its_tenant",
            "acceptance_target_and_organization_are_qualified_before_revision_conflict",
            "failed_revocation_audit_rolls_back_member_and_issued_invites",
            "accepted_receipt_retry_checks_authority_without_restoring_revoked_access",
            "reactivated_inviter_cannot_reuse_revoked_grant_but_can_issue_a_fresh_one",
        ):
            self.assertIn(name, self.authority_tests, name)


if __name__ == "__main__":
    unittest.main()
