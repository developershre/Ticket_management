# TMS API Reference (v1)

All actions use the deployed Apps Script Web App `/exec` URL with `POST` and `Content-Type: application/json`. `health.check` is public; all other actions require `auth.api_key`.

## Common request envelope

```json
{
  "version": "v1",
  "action": "reminders.create",
  "auth": { "api_key": "{{api_key}}" },
  "request_id": "client-generated-id",
  "data": {}
}
```

## Reminder actions

### `reminders.create`

Required: `employee_code`, `service_category`, `reminder_message`, `send_at`. The employee name and email are looked up from the `Employee` sheet. Use ISO 8601 timestamps with an explicit offset, for example `2026-11-03T10:00:00+05:30`.

- Past/current `send_at`: saves the row, then attempts to email immediately using the secure reminder form link.
- Future `send_at`: saves the row and schedules delivery through the five-minute trigger.
- If immediate sending fails, the reminder is still created; inspect `data.delivery.error` and retry using `reminders.sendNow` or allow the scheduled worker to retry.

Example: 

```json
{
  "version": "v1",
  "action": "reminders.create",
  "auth": { "api_key": "{{api_key}}" },
  "data": {
    "employee_code": "EMP001",
    "service_category": "IT Support",
    "reminder_message": "Please create a ticket for the network issue.",
    "send_at": "2026-11-03T10:00:00+05:30"
  }
}
```

### `reminders.sendNow`

Required: `reminder_id`. Sends the same HTML/plain-text email containing the secure `Create Support Ticket` link. It bypasses the scheduled time but respects the once-per-day duplicate guard. Add `"force": true` only when intentionally sending another email on the same day.

```json
{
  "version": "v1",
  "action": "reminders.sendNow",
  "auth": { "api_key": "{{api_key}}" },
  "data": { "reminder_id": "REM-..." }
}
```

### Other reminder actions

- `reminders.get`: `{ "reminder_id": "REM-..." }`
- `reminders.list`: optional filters `employee_code`, `status`, `service_category`, `query`, `offset`, `limit`.
- `reminders.update`: `reminder_id` plus any of `employee_code`, `service_category`, `reminder_message`, `send_at`, `status`. `status` may be `Pending` or `Cancelled`; completion is system-managed.

## Ticket and employee actions

- Tickets: `tickets.create`, `tickets.get`, `tickets.list`, `tickets.update`, `tickets.delete`.
- Employees: `employees.create`, `employees.get`, `employees.list`, `employees.update`, `employees.delete`.
- Run `health.check` without auth to confirm the deployed web app responds.

Refer to the Postman collection for complete request examples. The exact required sheet headers are defined in `MAIN.gs`.

## Required setup for email delivery

1. Run `initializeTMS()` from the Apps Script editor. It creates/validates `Tickets`, `Employee`, and `Reminders` and installs a five-minute sender trigger plus the edit trigger.
2. Run `authorizeReminderEmailService()` manually once and approve the MailApp permission prompt.
3. Run `generateTMSApiKey()` and place the generated value in the Postman collection variable `api_key`.
4. Deploy the project as a Web App, executing as the account that owns/has access to the spreadsheet and is authorized to send email. Deploy a new version after code changes.
5. Confirm `ScriptApp.getService().getUrl()` returns the deployed `/exec` URL; the reminder email link uses `?view=reminder&token=...`.

Apps Script time triggers are approximate; every five minutes means delivery occurs on the first trigger execution after the scheduled date/time, not exactly at the specified second.

## Delivery troubleshooting

- Check the API response `success`, `data.delivery.email_sent`, and `data.delivery.error`. A reminder can be saved successfully even when an immediate email attempt fails.
- Open Apps Script **Executions** and inspect failed `doPost` or `sendDueReminderEmails` executions.
- Check `MailApp.getRemainingDailyQuota()` via `authorizeReminderEmailService()` and verify the employee's `Mail` field.
- `MailApp.sendEmail` accepting a message means Apps Script handed it to Gmail; it does not guarantee inbox placement. Check spam/quarantine and mail routing if execution succeeds but the recipient sees nothing.

## Security

The API key is a project-level secret. Do not embed it in public HTML or distributed untrusted clients. Reminder URLs are bearer links: anyone possessing a valid pending reminder link can submit a ticket for that reminder. Tokens are stored hashed in Script Properties and invalidated when the reminder is completed/cancelled.

The code must be verified in your Apps Script account with a controlled test before production use; this package has not been run against your live spreadsheet or mailbox.
