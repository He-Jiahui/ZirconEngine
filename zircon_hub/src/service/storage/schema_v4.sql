ALTER TABLE invitations ADD COLUMN inviter_issuer TEXT;
ALTER TABLE invitations ADD COLUMN inviter_subject TEXT;
-- Older records cannot prove who issued their grant. Retain history, revoke pending grants.
UPDATE invitations SET status='revoked' WHERE status='pending';
CREATE INDEX invitation_sender ON invitations(organization_id,inviter_issuer,inviter_subject,status);
PRAGMA user_version = 4;
