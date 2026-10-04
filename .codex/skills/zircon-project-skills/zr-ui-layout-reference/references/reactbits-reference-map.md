# ReactBits Pro reference map

This is the approved external reference set for ZirconEngine layouts. The URLs originate from the user's functional-design brief. The short descriptions were checked against the live ReactBits Pro catalogue on 2026-09-18; re-open a page when a detail matters because the catalogue can evolve. Use the links for structure and interaction semantics only—never copy Pro source, screenshots, or license-gated assets into this repository.

## Catalogue entry points

- [Blocks](https://pro.reactbits.dev/docs/blocks) — 22 section categories / 280 variants of animated composition patterns for hero, feature, bento, social proof, contact, navigation, auth, pricing, and related page sections.
- [Application UI](https://pro.reactbits.dev/docs/app-ui) — 38 categories / 300 blocks covering AI/agents, navigation, data, forms, overlays, auth/onboarding, and workflows.

Use Blocks when the question is “how should a page section compose?” Use Application UI and the detailed entries below when the question is “how should an interactive product surface behave?” A page may use one of each, but each region still needs an explicit primary reference.

## AI chat, agent activity, and tools

| Reference | Capability to borrow | Zircon use |
| --- | --- | --- |
| [AI Chat 9](https://pro.reactbits.dev/docs/app-ui/ai-chat/ai-chat-9) | Provider/model picker with a context-and-cost detail card and reasoning-effort control. | Model selection surfaces, provider comparison, and a clear place for cost/context metadata. |
| [AI Chat 7](https://pro.reactbits.dev/docs/app-ui/ai-chat/ai-chat-7) | File/context list with pinned sources and per-message citations that map answers to evidence. | File/source references, evidence affordances, citation disclosure, and a stable source list. |
| [AI Chat 2](https://pro.reactbits.dev/docs/app-ui/ai-chat/ai-chat-2) | Generated-artifact side panel with code/document previews, file entries, and version tabs. | File-list/detail split, tabbed artifact windows, preview/detail panes, and version history. |
| [AI Chat 5](https://pro.reactbits.dev/docs/app-ui/ai-chat/ai-chat-5) | Inline approval request that gates a destructive agent action. | In-context permission gates; keep impact and decision controls adjacent to the action. |
| [AI Chat 6](https://pro.reactbits.dev/docs/app-ui/ai-chat/ai-chat-6) | Multi-agent thread with per-agent identity, handoffs, a tool/request list, and a live activity rail. | Subagent identity, tool-call/request detail lists, handoff chronology, and a persistent activity rail. |
| [AI Chat 4](https://pro.reactbits.dev/docs/app-ui/ai-chat/ai-chat-4) | Model picker, live context/token meter, and conversation-settings popover. | Context budget and conversation controls without hiding them in a separate page. |
| [AI Chat 3](https://pro.reactbits.dev/docs/app-ui/ai-chat/ai-chat-3) | Compact embedded assistant with empty state, suggestions, and docked composer. | Embedded/mobile assistant panels and first-use prompts. |
| [AI Chat 1](https://pro.reactbits.dev/docs/app-ui/ai-chat/ai-chat-1) | Full thread with reasoning disclosure, live tool call, streaming answer, sources, and attachments. | Complete assistant conversation, file references, progressive disclosure, and streaming state. |
| [Prompt Input 3](https://pro.reactbits.dev/docs/app-ui/prompt-input/prompt-input-3) | Empty-session prompt launcher with suggestion chips and recent prompt history. | Blank-session/empty-state composition and prompt discovery. |
| [Agent Activity 7](https://pro.reactbits.dev/docs/app-ui/agent-activity/agent-activity-7) | Run detail panel with inputs, outputs, retries, and an error surface. | Session-end/detail views and recoverable failure presentation. |
| [Agent Activity 6](https://pro.reactbits.dev/docs/app-ui/agent-activity/agent-activity-6) | Parallel agent lanes showing concurrent workers, queue depth, and throughput. | Subagent implementation pages and concurrent-work visualization. |
| [Agent Activity 5](https://pro.reactbits.dev/docs/app-ui/agent-activity/agent-activity-5) | Terminal-style log stream with severity filters and an autoscroll pin. | Runtime terminal output, log filters, follow/pause behavior, and bounded scroll. |
| [Agent Activity 4](https://pro.reactbits.dev/docs/app-ui/agent-activity/agent-activity-4) | Run-history table with status, duration, cost, and rerun actions. | Historical request/run output and retry affordances. |
| [Agent Activity 3](https://pro.reactbits.dev/docs/app-ui/agent-activity/agent-activity-3) | Compact status rail with current action, progress, and stop control. | Persistent workflow status in a narrow rail or header slot. |
| [Agent Activity 2](https://pro.reactbits.dev/docs/app-ui/agent-activity/agent-activity-2) | Collapsible run timeline with nested steps and duration bars. | Workflow hierarchy, nested substeps, and elapsed-time disclosure. |
| [Agent Activity 1](https://pro.reactbits.dev/docs/app-ui/agent-activity/agent-activity-1) | Live activity stream with per-step status dots, elapsed timers, and cancel. | Real-time workflow operation visualization and cancellation. |
| [Tool Calls](https://pro.reactbits.dev/docs/app-ui/tool-calls) | Catalogue of invocation cards, results, diffs, and permission surfaces. | Tool-call list/detail information architecture. |
| [Tool Calls 2](https://pro.reactbits.dev/docs/app-ui/tool-calls/tool-calls-2) | Pending, success, and failure call rows with retry affordances. | Tool-call feedback states and recovery actions. |
| [Tool Calls 5](https://pro.reactbits.dev/docs/app-ui/tool-calls/tool-calls-5) | Code-execution card with stdout/stderr tabs and an exit-code badge. | Terminal result layout and output-channel switching. |
| [Tool Calls 4](https://pro.reactbits.dev/docs/app-ui/tool-calls/tool-calls-4) | Web-search result with ranked source cards and relevance metadata. | Web-access result cards and source ranking. |
| [Tool Calls 3](https://pro.reactbits.dev/docs/app-ui/tool-calls/tool-calls-3) | File-edit result rendered as a diff with added/removed line counts. | Patch presentation and concise change statistics. |
| [Agent Approval](https://pro.reactbits.dev/docs/app-ui/agent-approval) | Human-in-the-loop confirmation, permission, and audit surfaces. | Permission request, approval queue, audit history, and risk disclosure. |
| [Agent Plan](https://pro.reactbits.dev/docs/app-ui/agent-plan) | Task plans, checklists, and execution trees for multi-step work. | Plan/goal views, dependency trees, and step status. |
| [AI Usage](https://pro.reactbits.dev/docs/app-ui/ai-usage) | Token spend, quotas, rate limits, and per-model cost attribution. | Usage visualization, budget warnings, and model-level accounting. |

## App shell, navigation, and responsive chrome

| Reference | Capability to borrow | Zircon use |
| --- | --- | --- |
| [App Shell 4](https://pro.reactbits.dev/docs/app-ui/app-shell/app-shell-4) | Three-column frame with a toggleable context panel beside main content. | Hub/editor shell variants with optional detail/context columns. |
| [App Shell](https://pro.reactbits.dev/docs/app-ui/app-shell) | Family of complete frames pairing navigation, headers, and content regions. | Choose the closest frame before adding page-specific regions. |
| [App Sidebar](https://pro.reactbits.dev/docs/app-ui/app-sidebar) | Sidebar, icon rail, secondary panel, workspace switcher, and mobile drawer patterns. | Session/project list sidebars, collapsed rails, and drawer transitions. |
| [Command Menu](https://pro.reactbits.dev/docs/app-ui/command-menu) | Command palettes, quick search, nested pages, inline arguments, and keyboard hints. | Command/search UI and keyboard-first action discovery. |
| [Navbar](https://pro.reactbits.dev/docs/app-ui/navbar) | Top bars, workspace/project switchers, breadcrumbs, tabs, popovers, toolbar actions, and compact text/settings controls. | Hub top bar, editor chrome, scope selectors, settings entry points, text-composition controls, and tabbed headers. |
| [Mobile](https://pro.reactbits.dev/docs/app-ui/mobile) | Bottom tab bars, floating docks, expanding actions, sheets, and full-screen menus. | Small-window/mobile adaptations; preserve task priority while changing chrome. |

## Data display and utility surfaces

| Reference | Capability to borrow | Zircon use |
| --- | --- | --- |
| [Cards](https://pro.reactbits.dev/docs/app-ui/card) | Card galleries and standalone cards for people, plans, activity, and files. | Project cards, metric cards, catalog tiles, and compact summaries. |
| [Data Table](https://pro.reactbits.dev/docs/app-ui/data-table) | Toolbars, filters, selection, inline editing, sticky headers, and detail panels. | Project Browser tables, build history, bulk actions, and master/detail rows. |
| [Dashboard](https://pro.reactbits.dev/docs/app-ui/dashboard) | Metric grids, chart panels, status boards, and usage overviews. | Dashboard landing pages and operational summaries. |
| [Analytics](https://pro.reactbits.dev/docs/app-ui/analytics) | Charts, metric strips, funnels, cohorts, legends, and reporting surfaces. | Runtime/build analytics when a chart is actually required; keep data semantics explicit. |
| [List](https://pro.reactbits.dev/docs/app-ui/list) | Row lists, feeds, queues, trees, selectable collections, and expandable rows. | Catalogs, activity feeds, asset trees, and request queues. |
| [Filtering](https://pro.reactbits.dev/docs/app-ui/filtering) | Facets, query builders, filter drawers, derived counts, and refinement controls. | Search/filter toolbars and responsive filter drawers. |
| [File Manager](https://pro.reactbits.dev/docs/app-ui/file-manager) | Drive/file browser, folder tree/grid, detail preview, selection bar, and upload progress. | Asset/project browsing, file references, drag/drop, and transfer feedback. |
| [Monitoring](https://pro.reactbits.dev/docs/app-ui/monitoring) | Live metrics, service health, alert/incident views, logs, uptime, and error budgets. | Build/runtime monitoring, long-running task health, and alert timelines. |
| [Empty State](https://pro.reactbits.dev/docs/app-ui/empty-state) | First-run, no-result, all-caught-up, load-failure, and dropzone states. | Every collection and task surface must have an intentional empty/loading/error path. |

## Settings, forms, overlays, and feedback

| Reference | Capability to borrow | Zircon use |
| --- | --- | --- |
| [Settings Form](https://pro.reactbits.dev/docs/app-ui/settings-form) | Section navigation, profile/member/API-key forms, danger zones, and an unsaved-changes bar. | Hub Settings and any persistent configuration workflow. |
| [Forms](https://pro.reactbits.dev/docs/app-ui/forms) | Sectioned record forms, validation, review states, derived summaries, and completion meters. | New Project, project/configuration forms, and multi-step authoring. |
| [App Dialog](https://pro.reactbits.dev/docs/app-ui/app-dialog) | Dialogs, drawers, sheets, popovers, menus, scrollable bodies, and sticky footers. | Create/delete dialogs, detail sheets, menus, and responsive overlays. |
| [Notifications](https://pro.reactbits.dev/docs/app-ui/notifications) | Notification centers, banners, toasts, activity inboxes, and delivery preferences. | Hub snackbar/status banner, activity history, and persistent feedback. |

## Auth, onboarding, and operational workflows

| Reference | Capability to borrow | Zircon use |
| --- | --- | --- |
| [Onboarding](https://pro.reactbits.dev/docs/app-ui/onboarding) | Steppers, setup checklists, invitations, use-case pickers, and completion summaries. | First-run project setup or account/workspace activation. |
| [Paywall](https://pro.reactbits.dev/docs/app-ui/paywall) | Content locks, feature gates, quota notices, plan comparison, and upgrade prompts. | Capability/plan gates; keep scope and consequence copy explicit. |
| [Authentication](https://pro.reactbits.dev/docs/app-ui/authentication) | Sign-in, sign-up, provider selection, verification, workspace picker, and recovery. | Authenticated Hub entry and account/workspace switching. |
| [Kanban](https://pro.reactbits.dev/docs/app-ui/kanban) | Status columns, swimlanes, drag/drop, WIP/capacity, and board detail panels. | Build/task workflow boards when a board model is justified. |
| [Scheduling](https://pro.reactbits.dev/docs/app-ui/scheduling) | Calendars, agendas, availability editors, time pickers, and booking surfaces. | Planned jobs, release windows, or timed workflow scheduling. |

### Variant anchors used by the current ZUI fixtures

When a fixture names a numbered variant, keep the variant link in the mapping record so the information architecture and state choice can be rechecked instead of inferred from a screenshot:

- Authentication/onboarding: [authentication-1](https://pro.reactbits.dev/docs/app-ui/authentication/authentication-1), [authentication-6](https://pro.reactbits.dev/docs/app-ui/authentication/authentication-6), [onboarding-4](https://pro.reactbits.dev/docs/app-ui/onboarding/onboarding-4), and [onboarding-7](https://pro.reactbits.dev/docs/app-ui/onboarding/onboarding-7).
- Settings workflow: [settings-form-1](https://pro.reactbits.dev/docs/app-ui/settings-form/settings-form-1), [forms-1](https://pro.reactbits.dev/docs/app-ui/forms/forms-1), [app-dialog-1](https://pro.reactbits.dev/docs/app-ui/app-dialog/app-dialog-1), and [notifications-3](https://pro.reactbits.dev/docs/app-ui/notifications/notifications-3).
- Board and scheduling workflow: [kanban-1](https://pro.reactbits.dev/docs/app-ui/kanban/kanban-1), [kanban-5](https://pro.reactbits.dev/docs/app-ui/kanban/kanban-5), [scheduling-1](https://pro.reactbits.dev/docs/app-ui/scheduling/scheduling-1), and [scheduling-7](https://pro.reactbits.dev/docs/app-ui/scheduling/scheduling-7).

## Selection rule for compound pages

Use the shell/navigation link to establish the outer frame, then choose the smallest data or behavior link for each major region. For example, a project browser can combine **App Shell** + **Data Table** + **Filtering** + **Empty State**; an agent run page can combine **AI Chat 1** + **Agent Activity 2** + **Tool Calls 2** + **Agent Approval**. Keep the mapping explicit and do not add unrelated reference families just to make a page look busier.
