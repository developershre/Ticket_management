# TMS App — Work Log & Next Steps

Last updated: 2026-11-04

---

## 1. Ground rules (do not violate)

1. **The Google Sheet is the single source of truth.** SQLite (`management.db`) is a
   disposable local cache. Any local row can be deleted and rebuilt from the sheet.
2. **Never invent API fields or endpoints.** The API rejects unknown fields with
   `UNKNOWN_FIELD`. If an operation does not exist (e.g. deleting a reminder, or
   fetching notes), document it instead of inventing a call.
3. **Never mark an operation synced before the server confirms it.** Failed ops stay
   in `sync_queue` as `pending`/`failed`, never silently dropped.
4. **Send only documented fields.** Local-only bookkeeping columns
   (`server_ticket_id`, `server_last_updated`, `server_reminder_id`,
   `server_last_updated`) must never appear in an outbound payload.
5. **No baked-in seed employee data.** Employee codes/emails come from the sheet
   cache, from the user's form input, or from `TMS_EMPLOYEE_CODE`. Never hardcoded.
6. **Local DB shape mirrors the sheet headers** (`TMS_SHEET_HEADERS_README.md`).
7. **Do not revert the reminders sync work** (committed at `b10e111`).

Credentials resolution order (implemented in `src/api.rs`):
`TMS_*` env vars → `.env` file → `TMS_API.postman_collection.json` → built-in default.

Current `.env` (gitignored, do not commit):

```text
TMS_API_BASE_URL=https://script.google.com/macros/s/AKfycbzx_5xVNedTPoIiT6WJ68zxKKaaaH-YahglFcL81LnD206RqnhonLqqBkYgyMSbBhDaoA/exec
TMS_API_KEY=8c753c054f9f4655b000bee6dab0300c3eee2f3a178441eaa88133b029cfda10f6ddb6cd7ea04b39af3a699ebfe4636c
```

> The old key `902038705cda…` is **dead** (`UNAUTHORIZED`). `DEFAULT_API_KEY` in
> `src/api.rs:43` still holds the old key — it is only a last-resort fallback and is
> never used while `.env` exists. Decide whether to update or remove it.

---

## 2. Current state (verified 2026-11-04)

| Check | Result |
|---|---|
| `cargo build` | ✅ clean, no warnings |
| `--sync-once` | ✅ `state=Idle pending=0 conflicts=0 detail=Up to date` |
| `tickets` pull | ✅ 2 rows (`TKT-2026-000023`, `TKT-2026-000024`) |
| `employees` pull | ✅ 325 rows |
| `reminders` pull | ✅ **now works** (0 rows on the sheet) — no more `SCHEMA_MISMATCH` |
| `sync_queue` / `sync_conflicts` | 0 / 0 |
| Local `reminders` columns | 15 = 13 sheet headers + 2 local mapping columns |
| Reminder create API | ❌ blocked by a **sheet data problem** (see §5) |

Local DB counts: `tickets=2, employees=325, reminders=0, notes=0, queue=0, conflicts=0`.

---

## 3. What was done earlier (before this session)

- Built the Sheets ↔ SQLite sync layer: `src/api.rs`, `src/sync.rs`,
  `src/sync_test.rs`, plus queue/meta/conflict/mapping tables in `src/db.rs`.
- Enqueued all 12 local mutation sites, added the `SyncPill` status UI and the
  `--sync-once` / `--sync-test offline|resume|online` CLI flags.
- Added the real employee directory feature: `Employee` + `employees_list()`,
  `pull_employees` (1800 s gate), `invalidate_employee_cache`, employee cache in
  `USERS`, `src/pages/employees.rs` at `/employees`, sidebar nav + live badges.
- Removed every demo seeder, including `seed_employees_if_empty` and the giant
  `INITIAL_SHEET_EMPLOYEES_CSV` constant (uncommitted removal in this tree).
- Rebuilt the `USERS` schema (`ensure_user_schema`): nullable/duplicate emails,
  nullable `emp_code`, `division` column.
- Wiped the local DB and re-synced from the sheet (backup:
  `/tmp/opencode/management.db.bak-20261103`).

---

## 4. What was done in this session (2026-11-04)

### 4.1 Repaired the broken build

The working tree did not compile: a previous edit removed
`DbReminder::server_reminder_id` / `server_last_updated` while
`src/pages/reminders.rs` still used them.

**Decision:** keep the two mapping columns (the user said either way is fine).
Without them the app cannot target `reminders.update` / `reminders.sendNow` or
match pulled rows. They are documented as **local-only, never sent**.

- `src/db.rs` — restored both fields on `DbReminder` with doc comments.
- `cargo check` → ✅.

### 4.2 Removed the static seed-employee data

| File | Was | Now |
|---|---|---|
| `src/db.rs` `NewReminderInput::default` | `"11135"`, `"IT Support"`, `"it.support@bpsl.net"` | empty strings |
| `src/db.rs` `NewTicketInput::default` | `"11135"`, `"it.support@bpsl.net"`, `customer: "Internal"` | empty strings |
| `src/db.rs` `get_all_notes` | `COALESCE(emp_code, 'EMP-1001')` | `COALESCE(emp_code, '')` |
| `src/pages/notes.rs` | `create_note("EMP-1001", …)` | `create_note(&db::configured_employee_code(), …)` |
| `src/components/fab.rs` | `create_note("EMP-1001", …)` | same |
| `src/components/fab.rs` quick ticket | `employee_code: "11135"`, `email "it.support@bpsl.net"`, `customer "Internal"` | new **Employee Code** field (prefilled from `TMS_EMPLOYEE_CODE`), email + customer derived from the sheet cache |
| `src/pages/tickets.rs` quick-add | `"EMP-1001 (CurrentUser)"`, `"it.support@bpsl.net"`, `"Internal"` | configured employee + cache-derived email/customer, with a clear error toast when no employee is configured |
| `src/sync_test.rs` | hardcoded `"10081"` fallback | configured code → first cached sheet code → explicit error |

New helpers in `src/db.rs`:

- `configured_employee_code()` — `TMS_EMPLOYEE_CODE` (env → `.env` → Postman), else `""`.
- `employee_name(code)` / `employee_email(code)` — read-only lookups in the `USERS` cache.
- `first_cached_employee_code()` — test-harness fallback only.

### 4.3 Reminder correctness fixes

- `src/api.rs` — `reminders_list` no longer tries the non-existent
  `reminders.listAll` (it returned `UNKNOWN_ACTION` on every pull). It now calls
  `reminders.list` directly.
- `src/api.rs` — `ServerReminder` gained `cc_emails` (sheet column M,
  `CC Emails`), tolerant of string **or** array via the new `flex_list_str`.
- `src/db.rs` — `reminders` table gained `cc_emails` (migrated in place by
  `ensure_reminder_sync_columns`), and pull/insert/apply now carry it.
- `src/sync.rs` — outbound reminder payloads are whitelisted to documented fields:
  - create → `employee_code`, `service_category`, `reminder_message`, `send_at`,
    plus `cc_emails` only when non-empty;
  - update → those plus `status`, and `cc_emails` forwarded even when empty
    (that is how the API clears CC).
  - `employee_name`, `email_address`, `email_sent_at`, `ticket_id`, timestamps and
    the `server_*` mapping columns are **never** sent.
- `src/pages/reminders.rs` `send_now`:
  - requires a real sheet `Reminder ID` (no more sending the local `REM-<n>` id);
  - calls the API **first**, marks the local row sent only after success;
  - `SCHEMA_MISMATCH` now produces an actionable error instead of pretending it sent.
- `src/pages/reminders.rs` `delete_reminder`:
  - the API has **no** `reminders.delete` (documented 17 actions);
  - deleting now enqueues `status: "Cancelled"` for rows that reached the sheet
    (stops the scheduled email) and explains that the sheet keeps the row.

### 4.4 Ticket sync correctness

- `src/sync.rs` `with_employee_code` previously **overwrote** the employee code of
  every ticket with `TMS_EMPLOYEE_CODE` — including on updates, which silently
  reassigned tickets. It now fills only an empty code, and only on **creates**
  (`sanitize_ticket_payload(client, payload, for_create)`).

---

## 5. ⚠️ Blocking issue: the Employee sheet has Mail and Division swapped

Live probes against the deployed script:

```text
employees.listAll → {
  "employee_code": "10001",
  "employee_name": "AMIT SHARMA",
  "department": "FINANCE AND ACCOUNTS",
  "email": "SPONGE IRON DIVISION",          ← Division value
  "division": "aks@bhagwatipower.com"       ← Mail value
}
```

This is true for **all 325 rows** (0 rows have an `@` in `email`).

Consequence — every reminder create fails:

```text
reminders.create { employee_code: "10081", service_category, reminder_message, send_at }
→ EMPLOYEE_EMAIL_INVALID: "The employee does not have a valid email address."
```

Confirmed not caused by the app:

- without `cc_emails` → same error;
- `email_address` cannot be sent (`UNKNOWN_FIELD` — undocumented);
- the failure happens for `10001` as well as `10081`.

**Root cause is sheet-side:** the `Employee` tab has `Division` in column D and
`Mail` in column E, while the API reads those two positions as *Mail* then
*Division* (the standard layout documented in `TMS_SHEET_HEADERS_README.md` §2:
`Employee Code | Name | Dept | Mail | Division`).

### Two possible fixes (pick one, do it in the sheet)

**A. Reorder the sheet columns (recommended, permanent)**
1. Insert a new column E and move the current `Mail` column so the header row is
   exactly: `Employee Code | Name | Dept | Mail | Division`.
2. Verify with `employees.listAll` that `email` now contains `@` and `division`
   contains the division name.

**B. Swap the header labels** of columns D and E, then move the data to match.
Same end state as A.

After the sheet is fixed, no app rebuild is needed — the pull re-caches everything
(`ensure_user_schema` + `replace_employees`, full DELETE + re-insert).

> Until then, `reminders.create` / `reminders.update` / `reminders.sendNow` cannot
> be exercised end-to-end. `reminders.list` works and returns 0 rows.

### Optional app-side mitigation (not done yet)

`pull_employees` could defensively swap `email`/`division` when
`email` has no `@` and `division` does (safe: it no-ops once the sheet is fixed).
Pros: the employee page and reminder picker show real emails immediately.
Cons: it hides the sheet problem and would invert data on any employee
write-back. **Decide with the user before adding it.**

---

## 6. Next steps (in order)

1. **Fix the Employee sheet** (see §5, option A). This unblocks reminders.
2. **Re-sync and verify the cache:**
   ```bash
   cd /home/gamerrock/Documents/projects/management
   TMS_EMPLOYEE_CODE=10081 ./target/debug/management --sync-once
   sqlite3 management.db "SELECT emp_code, emp_email FROM USERS WHERE COALESCE(emp_email,'')<>'' LIMIT 5;"
   ```
   Expect real `@` addresses, `division` holding division names.
3. **End-to-end reminder test** (future `send_at` so no email fires):
   ```bash
   TMS_EMPLOYEE_CODE=<a real code with a valid email> ./target/debug/management
   ```
   - create a reminder in the UI → check `sync_queue` drains and the row appears in
     the sheet with `Employee Name` / `Email Address` / `Created At` filled by the server;
   - edit category / send-at → confirm `reminders.update` returns `success`;
   - `sendNow` on a pending reminder → confirm `delivery` in the response and that
     `Email Sent At` is written by the server;
   - then run `--sync-once` and confirm the pull applies the server values back.
4. **Decide the "mark complete" mapping.** The UI checkbox currently writes
   `status: "Completed"` locally and enqueues `Completed` — but the docs only allow
   `Pending` and `Cancelled` (`Completed` is system-managed). Test on a real row:
   - if `Completed` is rejected → change `toggle_done` to enqueue `Cancelled` when
     ticking and `Pending` when unticking, and treat `Cancelled` as "done" in
     `initial_reminders()` so the UI round-trips;
   - if accepted, keep it but note the dependency on undocumented behaviour.
5. **Cleanup after testing**: reminders cannot be deleted via the API — set them to
   `Cancelled` (and remove the local rows) instead of leaving test data behind.
6. **Commit** the work (the tree also contains the earlier, still-uncommitted
   seed-removal and schema changes). Suggested message:
   `fix: remove seed employee data, mirror sheet reminder schema, harden reminder sync`.
7. **Decide about credentials**: update or remove the dead `DEFAULT_API_KEY`
   (`src/api.rs:43`). `.env` is gitignored and holds the live key.

---

## 7. Verified API facts (live probes, 2026-11-04)

- Envelope: `POST` `/exec` with `{version:"v1", action, auth:{api_key}, request_id, data}`.
  HTTP is always 200; success/error lives in the body.
- `reminders.create` required: `employee_code`, `service_category`,
  `reminder_message`, `send_at`. Optional: `cc_emails` (array or
  comma/semicolon string).
- `reminders.create` rejects `email_address` → `UNKNOWN_FIELD`.
- `reminders.update` accepts: `employee_code`, `service_category`,
  `reminder_message`, `send_at`, `status`, `cc_emails`.
- Documented statuses: **`Pending` and `Cancelled` only**. `Completed` is
  system-managed; completed reminders cannot be updated.
- `reminders.sendNow`: `reminder_id` required, `force` optional; only pending
  reminders can be sent.
- `reminders.listAll` does **not** exist (`UNKNOWN_ACTION`).
- `employees.listAll` works and returns all 325 rows (no pagination).
- `tickets.create` requires `employee_code`, `category`, `subject`; the employee
  must exist in the `Employee` sheet.
- Sheet-side header gate: a 12-column `Reminders` tab causes `SCHEMA_MISMATCH`;
  the 13-column layout in `TMS_SHEET_HEADERS_README.md` (with `CC Emails` at M)
  is required — **now in place and verified**.
- Errors seen: `UNAUTHORIZED` (old key), `UNKNOWN_FIELD`, `UNKNOWN_ACTION`,
  `VALIDATION_ERROR`, `NOT_FOUND`, `EMPLOYE_NOT_FOUND`/`EMPLOYEE_EMAIL_INVALID`,
  `SCHEMA_MISMATCH`.

---

## 8. Reference map

| Path | Purpose |
|---|---|
| `src/api.rs` | HTTP client, `ApiConfig::load()` (env → `.env` → Postman), DTOs (`ServerTicket`, `ServerReminder`, `Employee`), `flex_str`/`flex_list_str` |
| `src/sync.rs` | background worker, pull cycles, payload sanitizers, enqueue helpers, immediate employee ops |
| `src/sync_test.rs` | `--sync-test offline\|resume\|online` harness (creates/deletes `SYNC-TEST*` rows) |
| `src/db.rs` | SQLite schema/migrations, CRUD, cache helpers (`configured_employee_code`, `employee_email`, `employee_name`, `first_cached_employee_code`) |
| `src/pages/reminders.rs` | reminder UI, create/toggle/sendNow/delete + enqueue sites |
| `src/pages/tickets.rs` | ticket board, create/edit, quick-add |
| `src/pages/employees.rs` | read-only employee directory (sheet cache) |
| `src/components/fab.rs` | quick-add dialog (note / ticket / reminder) |
| `src/components/app_sidebar.rs` | nav + live counts |
| `TMS_SHEET_HEADERS_README.md` | authoritative sheet tabs/headers (Tickets 23, Employee 5, Reminders 13) |
| `TMS_API_COMPLETE_README.md` | authoritative API reference (17 actions) |
| `TMS_API.postman_collection.json` | request examples + fallback credential source |

### Useful commands

```bash
cargo check / cargo build
./target/debug/management --sync-once
./target/debug/management --sync-test offline|resume|online
sqlite3 management.db "SELECT status, COUNT(*) FROM sync_queue GROUP BY status;"
sqlite3 management.db "SELECT * FROM sync_conflicts;"
```

The app itself is run with `dx serve` (auto-rebuild/restart).
