# CC Recipients and Reminder Digest Emails

## Optional CC recipients

The reminder API accepts `cc_emails` on `reminders.create` and `reminders.update`. It can be an array of addresses or a comma/semicolon-separated string. The system validates each address, removes duplicates, excludes the primary employee email from CC, and limits each reminder to 49 CC recipients.

The recipients are stored in the final `CC Emails` column on the `Reminders` sheet. If the field is omitted, no CC is used. Passing an empty string or empty array through `reminders.update` clears the CC list.

Example:

```json
{
  "version": "v1",
  "action": "reminders.create",
  "auth": { "api_key": "{{api_key}}" },
  "data": {
    "employee_code": "EMP001",
    "service_category": "IT Support",
    "reminder_message": "Please create a support ticket.",
    "send_at": "2026-12-01T10:00:00+05:30",
    "cc_emails": ["it.manager@example.com", "supervisor@example.com"]
  }
}
```

## Digest behavior

- Grouping is per employee code and recipient email address. Different employees are never combined into one email.
- On a scheduled run, if one employee has **more than five due, unsent, pending reminders**, they receive one digest email. Five or fewer reminders are sent as separate emails.
- Every reminder in a digest has its own secure `Create Support Ticket` URL and reminder ID. Submitting one item completes only that reminder.
- CC recipients are the deduplicated union of CC values across the reminders in that digest. Therefore, every CC recipient on a digest can see all reminder details in that digest. Configure CC with that visibility in mind.
- The digest updates `Email Sent At` and `Last Updated At` for every included reminder only after `MailApp.sendEmail` accepts the message.
- `reminders.sendNow` applies the same digest threshold to due pending reminders for the selected employee; `force: true` can deliberately resend the selected reminder.

## Upgrade

Back up the spreadsheet and Apps Script project. Replace the project files with this package, then run `initializeTMS()` once. The initializer adds `CC Emails` as the thirteenth/final reminder column without reordering the original twelve columns. Re-authorize MailApp only if Apps Script asks for it, and deploy a new Web App version.
