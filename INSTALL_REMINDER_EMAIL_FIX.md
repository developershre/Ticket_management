# Integrated reminder email fix

This package contains the full TMS Apps Script project with the reminder delivery fixes integrated.

## Install

1. Back up the current Apps Script project.
2. Replace the contents of each existing Apps Script file with the same-named file in this package. Create any missing `.gs` or HTML files. Do not retain duplicate function definitions.
3. Confirm only one `doGet(e)` (in `WEB_FORMS.gs`) and one `doPost(e)` (in `API.gs`) remain.
4. Run `initializeTMS()` manually and authorize spreadsheet/trigger permissions. It installs the five-minute `sendDueReminderEmails` trigger and the `onReminderSheetEdit` trigger.
5. Run `authorizeReminderEmailService()` manually and approve MailApp permissions.
6. Run `generateTMSApiKey()` if needed; update the key in clients/Postman. Do not regenerate unless intentionally rotating it.
7. Deploy > Manage deployments > Edit > New version > Deploy. The web app should execute as the spreadsheet-owning account (or another account that has the required access and email authorization).
8. Import `TMS_API.postman_collection.json`, set `base_url`, `api_key`, and use a real employee code.

## Email behavior

- `reminders.create`: creates the row. Past/current `send_at` values trigger an immediate send attempt. Future `send_at` values wait for the time-driven worker. The API response distinguishes `delivery.email_sent=true`, `delivery.status="scheduled"`, or a failure with `delivery.error`.
- `reminders.sendNow`: sends the selected pending reminder immediately using the secure form URL and the common email template. It avoids duplicates for the same calendar day unless `force:true` is explicitly supplied.
- `sendDueReminderEmails`: checks due reminders every five minutes; sends at most one per calendar day; completes reminders where a matching ticket already exists.
- A successful `MailApp.sendEmail` call means Apps Script accepted the send request, not that the recipient's inbox delivery is guaranteed. Check spam/quarantine and mail routing if delivery is not visible.

## Token behavior

Reminder tokens are random bearer tokens; only their SHA-256 hashes are stored in Script Properties. Token validation checks the stored hash record and reminder state, rather than requiring it to be the latest active mapping. Tokens are invalidated when the reminder is completed/cancelled. This prevents a newly generated token from invalidating an earlier link while the reminder is still pending.

## Verification

Run `runTMSSelfTest()` and inspect all checks. Then create a controlled reminder for an employee mailbox you can access. Use a past/current timestamp to test immediate delivery, or a future timestamp to test scheduled delivery. Inspect the API `delivery` object and Apps Script Executions logs. The package cannot be tested against your live Apps Script account from this environment.
