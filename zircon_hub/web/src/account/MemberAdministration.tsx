import CheckOutlinedIcon from "@mui/icons-material/CheckOutlined";
import CloseOutlinedIcon from "@mui/icons-material/CloseOutlined";
import EditOutlinedIcon from "@mui/icons-material/EditOutlined";
import PersonAddOutlinedIcon from "@mui/icons-material/PersonAddOutlined";
import RefreshOutlinedIcon from "@mui/icons-material/RefreshOutlined";
import SwapHorizOutlinedIcon from "@mui/icons-material/SwapHorizOutlined";
import { Alert, Box, Button, MenuItem, TextField, Typography } from "@mui/material";
import { useEffect, useRef, useState } from "react";
import { HubIconButton, HubSwitch } from "../components/inputs";
import { HubDialog } from "../components/overlays/HubDialog";
import { hubTokens } from "../theme/tokens";
import { useAccount } from "./context";
import type { AccountCopy } from "./copy";
import type { Member, MemberRole, Organization, OrganizationMutation } from "./protocol";

type Draft =
  | { kind: "member"; member: Member; role: MemberRole; active: boolean; revision: string }
  | { kind: "transfer"; member: Member; revision: string }
  | { kind: "invite"; issuer: string; subject: string; role: MemberRole; days: number; revision: string };

export function MemberAdministration({ organization, copy, disabled }: { organization: Organization; copy: AccountCopy; disabled: boolean }) {
  const state = useAccount();
  const { controller } = state;
  const account = state.snapshot.account;
  const [draft, setDraft] = useState<Draft | null>(null);
  const [refreshing, setRefreshing] = useState(false);
  const [reconfirmed, setReconfirmed] = useState(false);
  const attempt = useRef(0);
  const identity = JSON.stringify([account.generation, account.issuer, account.subject, organization.id]);
  useEffect(() => { attempt.current += 1; setDraft(null); setRefreshing(false); setReconfirmed(false); return () => { attempt.current += 1; }; }, [identity]);

  const actor = state.members.items.find(member => member.issuer === account.issuer && member.subject === account.subject);
  const permissionReady = state.members.loaded && !state.members.loading && !state.members.error;
  const owner = permissionReady && actor?.active && actor.role === "owner";
  const canInvite = permissionReady && actor?.active && (actor.role === "owner" || actor.role === "admin");
  const needsRefresh = state.organizationsNeedingRefresh.includes(organization.id);
  const policyChanged = draft !== null && draft.revision !== organization.policyRevision;
  const locked = disabled || refreshing || state.organizations.loading;
  const target = draft && draft.kind !== "invite" ? state.members.items.find(member => sameMember(member, draft.member)) : undefined;
  const targetEligible = draft?.kind === "invite" || Boolean(target && target.role !== "owner" && (draft?.kind !== "transfer" || target.active));
  const allowed = draft?.kind === "invite" ? canInvite : owner;
  const validIdentity = draft?.kind === "invite" ? validText(draft.issuer) && validText(draft.subject) : true;
  const duplicate = draft?.kind === "invite" && state.members.items.some(member => member.issuer === draft.issuer.trim() && member.subject === draft.subject.trim());
  const valid = Boolean(draft && validIdentity && !duplicate && targetEligible && allowed);
  const changed = draft?.kind !== "member" || draft.role !== target?.role || draft.active !== target?.active;

  const close = () => { attempt.current += 1; setDraft(null); setRefreshing(false); setReconfirmed(false); };
  const open = (value: Draft) => { attempt.current += 1; setReconfirmed(false); setDraft(value); };
  const refresh = async () => {
    if (!draft || refreshing) return;
    const ticket = ++attempt.current;
    setRefreshing(true);
    const next = await controller.refreshOrganization(draft.kind === "invite" ? undefined : draft.member);
    if (ticket !== attempt.current) return;
    setRefreshing(false);
    if (next) { setDraft(value => value ? { ...value, revision: next.policyRevision } : null); setReconfirmed(true); }
  };
  const submit = async () => {
    if (!draft || locked || !valid || !changed || needsRefresh || policyChanged) return;
    const ticket = attempt.current;
    let mutation: OrganizationMutation;
    if (draft.kind === "member") mutation = { action: "set-member", issuer: draft.member.issuer, subject: draft.member.subject, role: draft.role, active: draft.active };
    else if (draft.kind === "transfer") mutation = { action: "transfer-ownership", issuer: draft.member.issuer, subject: draft.member.subject };
    else mutation = { action: "invite", issuer: draft.issuer.trim(), subject: draft.subject.trim(), role: draft.role, expires_at: Math.floor(Date.now() / 1000) + draft.days * 86400 };
    const succeeded = await controller.mutate(organization.id, draft.revision, mutation);
    if (ticket !== attempt.current) return;
    const current = controller.getSnapshot();
    const unknown = current.mutation?.status === "unknown" || current.snapshot.operationsError || current.snapshot.operations.some(operation => operation.status === "unknown");
    if (succeeded || unknown) close();
  };
  const title = draft?.kind === "member" ? copy.editMember : draft?.kind === "transfer" ? copy.transferOwnership : copy.inviteMember;
  const confirm = draft?.kind === "member" ? copy.saveMember : draft?.kind === "transfer" ? copy.confirmTransfer : copy.createInvitation;

  return <>
    {canInvite ? <Box sx={{ display: "flex", justifyContent: "flex-end", mb: 0.5 }}>
      <Button size="small" startIcon={<PersonAddOutlinedIcon />} disabled={locked} onClick={() => open({ kind: "invite", issuer: account.issuer ?? "", subject: "", role: "member", days: 1, revision: organization.policyRevision })}>{copy.inviteMember}</Button>
    </Box> : null}
    {needsRefresh && !draft ? <Alert severity="warning" sx={{ my: 1, "& .MuiAlert-message": { minWidth: 0 } }}>{copy.refreshRequired}
      <Button size="small" startIcon={<RefreshOutlinedIcon />} disabled={state.mutation?.status === "running" || refreshing} onClick={() => void controller.refreshOrganization()}>{copy.refreshOrganization}</Button>
    </Alert> : null}
    {state.members.items.map(member => <Box key={JSON.stringify([member.issuer, member.subject])} data-testid={`account-member-${member.subject}`} sx={{ display: "flex", flexWrap: "wrap", alignItems: "center", gap: 0.5, py: 0.8, borderBottom: `1px solid ${hubTokens.colors.line}`, minWidth: 0 }}>
      <Box sx={{ flex: "1 1 170px", display: "grid", gap: 0.2, minWidth: 0 }}>
        <Typography variant="body2" sx={wrap}>{member.subject}</Typography>
        <Typography variant="caption" color="text.secondary" sx={wrap}>{member.issuer}</Typography>
        <Typography variant="caption" color={member.active ? "text.secondary" : "warning.main"}>{copy.roles[member.role]}{member.active ? "" : ` · ${copy.inactive}`}</Typography>
      </Box>
      {owner && member.role !== "owner" ? <Box sx={{ display: "flex", flexWrap: "wrap", alignItems: "center", gap: 0.5, maxWidth: "100%" }}>
        <HubIconButton label={`${copy.editMember}: ${member.subject}`} disabled={locked} sx={{ width: 34, height: 34 }} onClick={() => open({ kind: "member", member, role: member.role as MemberRole, active: member.active, revision: organization.policyRevision })}><EditOutlinedIcon fontSize="small" /></HubIconButton>
        {member.active ? <Button size="small" startIcon={<SwapHorizOutlinedIcon />} disabled={locked} sx={{ maxWidth: "100%", whiteSpace: "normal" }} onClick={() => open({ kind: "transfer", member, revision: organization.policyRevision })}>{copy.transferOwnership}</Button> : null}
      </Box> : null}
    </Box>)}
    <HubDialog open={draft !== null} title={title} onClose={close} actions={<Box sx={{ display: "flex", flexWrap: "wrap", justifyContent: "flex-end", gap: 1, p: 1, width: "100%" }}>
      <Button startIcon={<CloseOutlinedIcon />} onClick={close}>{copy.closeDialog}</Button>
      {needsRefresh || policyChanged ? <Button startIcon={<RefreshOutlinedIcon />} disabled={refreshing || state.mutation?.status === "running"} onClick={() => void refresh()}>{copy.refreshOrganization}</Button>
        : <Button variant="contained" color={draft?.kind === "transfer" ? "warning" : "primary"} startIcon={draft?.kind === "transfer" ? <SwapHorizOutlinedIcon /> : <CheckOutlinedIcon />} disabled={locked || !valid || !changed} onClick={() => void submit()}>{confirm}</Button>}
    </Box>}>
      {draft ? <Box sx={{ display: "grid", gap: 1.5, pt: 0.5, minWidth: 0 }}>
        <Typography variant="subtitle2" sx={wrap}>{organization.name}</Typography>
        {needsRefresh || policyChanged ? <Alert severity="warning">{needsRefresh ? copy.refreshRequired : copy.policyChanged}</Alert> : reconfirmed ? <Alert severity="info">{copy.confirmAgain}</Alert> : null}
        {permissionReady && !allowed ? <Alert severity="warning">{copy.permissionUnavailable}</Alert> : !targetEligible && permissionReady ? <Alert severity="warning">{copy.memberUnavailable}</Alert> : null}
        {draft.kind !== "invite" ? <Box sx={{ display: "grid", gap: 0.3, minWidth: 0 }}>
          <Typography variant="body2" sx={wrap}>{draft.member.subject}</Typography><Typography variant="caption" color="text.secondary" sx={wrap}>{draft.member.issuer}</Typography>
        </Box> : <>
          <TextField label={copy.identityIssuer} value={draft.issuer} size="small" disabled={locked} error={Boolean(draft.issuer) && !validText(draft.issuer)} helperText={Boolean(draft.issuer) && !validText(draft.issuer) ? copy.identityInvalid : undefined} onChange={event => setDraft({ ...draft, issuer: event.target.value })} slotProps={{ htmlInput: { maxLength: 256 } }} />
          <TextField label={copy.identitySubject} value={draft.subject} size="small" disabled={locked} error={Boolean(draft.subject) && (!validText(draft.subject) || duplicate)} helperText={duplicate ? copy.duplicateMember : Boolean(draft.subject) && !validText(draft.subject) ? copy.identityInvalid : undefined} onChange={event => setDraft({ ...draft, subject: event.target.value })} slotProps={{ htmlInput: { maxLength: 256 } }} />
        </>}
        {draft.kind === "transfer" ? <Alert severity="warning">{copy.transferWarning}</Alert> : <TextField select label={copy.role} size="small" value={draft.role} disabled={locked} onChange={event => setDraft({ ...draft, role: event.target.value as MemberRole })}>
          {(["admin", "member", "viewer"] as const).map(role => <MenuItem key={role} value={role}>{copy.roles[role]}</MenuItem>)}
        </TextField>}
        {draft.kind === "member" ? <HubSwitch label={copy.active} checked={draft.active} disabled={locked} onChange={active => setDraft({ ...draft, active })} /> : null}
        {draft.kind === "invite" ? <TextField select label={copy.expiresIn} size="small" value={draft.days} disabled={locked} onChange={event => setDraft({ ...draft, days: Number(event.target.value) })}>
          <MenuItem value={1}>{copy.oneDay}</MenuItem><MenuItem value={3}>{copy.threeDays}</MenuItem><MenuItem value={7}>{copy.sevenDays}</MenuItem>
        </TextField> : null}
      </Box> : null}
    </HubDialog>
  </>;
}

const wrap = { overflowWrap: "anywhere", minWidth: 0 } as const;
function sameMember(left: Member, right: Member) { return left.issuer === right.issuer && left.subject === right.subject; }
function validText(value: string) { return value.trim().length > 0 && new TextEncoder().encode(value.trim()).length <= 256 && !/[\x00-\x1f\x7f-\x9f]/.test(value); }
