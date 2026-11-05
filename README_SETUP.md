# TMS Updated Apps Script Project

## Files
- MAIN.gs: TMS configuration, initialization, sheet setup and menus.
- API.gs: authenticated JSON API, ticket/employee/reminder CRUD and send-now.
- REMINDERS.gs: secure reminder tokens, scheduled emails and triggers.
- WEB_FORMS.gs: Web App routes and ticket form handlers.
- ATTACHMENTS.gs: Drive attachment storage.
- NewTicket.html: public ticket form.
- ReminderTicket.html: secure reminder ticket form.
- TESTS.gs: read-only diagnostics and an explicit live-email test.

## Installation
1. In Apps Script, create one file for each `.gs` and HTML file above and paste the matching contents.
2. Remove old duplicate functions/files. There must be exactly one `doGet`, one `doPost`, one `getTicketFormOptions`, one `formatReminderSheet_`, and one `sendDueReminderEmails`.
3. Set the spreadsheet timezone and Apps Script project timezone to Asia/Kolkata (or the timezone your business uses).
4. Run `initializeTMS()` and authorize the requested Google permissions. This initializes/validates the three sheets and installs the five-minute reminder worker plus the sheet-edit trigger.
5. Run `authorizeReminderEmailService()` manually once from the Apps Script editor and approve MailApp permissions. This is required before API or trigger executions can send mail.
6. Run `generateTMSApiKey()`. Copy the generated key from the execution log and store it securely; it is not saved in plaintext.
7. Deploy > New deployment > Web app. Choose the execution identity and access policy appropriate to your organization. Copy the deployed `/exec` URL.
8. Run `runTMSSelfTest()` and inspect the execution log/result.

## Testing
- `runTMSSelfTest()` checks sheets/headers, config, helper availability, Web App deployment, API-key configuration, mail quota, triggers, attachments helpers, and a temporary secure-token round trip. It does not create a ticket, upload a file, or send email.
- `testReminderEmailById("REM-...")` sends a real secure reminder email immediately to the recipient on that pending reminder. Use only when you intend to send an email. `authorizeReminderEmailService()` checks mail authorization and remaining recipient quota.
- To test the complete ticket flow, create a test reminder for an employee whose email you can access, run `testReminderEmailById`, open the link, submit the form, and verify the ticket row plus reminder status/Ticket ID.

## Reminder behavior
- Scheduled sender checks pending reminders every five minutes. Google time triggers are approximate; email is sent on the first worker run after `Send At`.
- A pending reminder is emailed at most once per calendar day by the scheduled worker. `reminders.create` sends immediately when `send_at` is in the past or present; future reminders wait for the worker. `reminders.sendNow` sends the same secure ticket-form email immediately, subject to the daily-send guard unless `force: true` is supplied.
- A matching ticket (same employee code and category, created on/after reminder creation) completes the reminder.
- A reminder form submission creates a ticket using employee/category values from the validated reminder, then marks the reminder Completed and invalidates its token.

## API reminder email behavior
- `reminders.create` persists the reminder first. For a due/past `send_at`, it attempts delivery immediately and returns a `delivery` object. For future `send_at`, it returns `delivery.status = "scheduled"`. If immediate mail fails, the reminder remains saved and the response includes `delivery.email_sent = false` plus the error.
- `reminders.sendNow` sends the same secure ticket-form email as the scheduled worker, not a diagnostic-only email. The optional `force: true` flag bypasses the once-per-day check.
- Both API and scheduled sends use the `?view=reminder&token=...` route and update `Email Sent At` only after `MailApp.sendEmail` accepts the message.

## Security notes
- The API key is required for API actions other than health check. Do not put it in public HTML.
- Reminder links are bearer links: anyone possessing the link can submit a ticket for that reminder until it is completed/cancelled.
- ATTACHMENTS.gs currently uses `ANYONE_WITH_LINK` sharing. Change `LINK_ACCESS` to `DOMAIN_WITH_LINK` or `PRIVATE` if public link access is not acceptable.
- Public `NewTicket.html` only verifies employee code plus employee email; for stronger identity assurance, add organization authentication or an OTP flow.

## Important
The source files were assembled from the files supplied in this conversation and the existing project archive. The code has not been executed inside your Google Apps Script account. Run diagnostics and a controlled end-to-end test before production use.
