# Install custom reminder schedule and shared ticket form

1. Back up your Apps Script project and spreadsheet.
2. Replace `MAIN.gs`, `API.gs`, `REMINDERS.gs`, `WEB_FORMS.gs`, and `NewTicket.html` with the files in this package. Keep the other project files unchanged.
3. Run `initializeTMS()` once. It adds `Repeat Frequency` as the 14th Reminders column without reordering existing columns. Existing blank values are treated as `daily`.
4. Set both spreadsheet and Apps Script project timezone to the organization timezone (e.g. `Asia/Kolkata`).
5. Run `authorizeReminderEmailService()` once and approve permissions.
6. Run `installReminderEmailTrigger()` once. It installs the five-minute schedule worker.
7. Deploy a new Web App version.
8. Import `TMS_API.postman_collection.json` into Postman and configure `base_url` and `api_key`.

## API schedule example

```json
{
  "version": "v1",
  "action": "reminders.create",
  "auth": { "api_key": "{{api_key}}" },
  "data": {
    "employee_code": "EMP001",
    "service_category": "IT Support",
    "reminder_message": "Please create a support ticket.",
    "send_at": "2026-11-06T10:30:00+05:30",
    "repeat_frequency": "daily",
    "cc_emails": []
  }
}
```

Update schedule example: `reminders.update` with `reminder_id`, `send_at`, and/or `repeat_frequency`. Only Pending reminders can be updated.

## Delivery semantics
- `daily`: send once per local calendar day at/after the configured time, starting on the configured date.
- `once`: send once at/after the configured date/time.
- The trigger runs every five minutes; exact-minute delivery is not guaranteed.
- If the secure reminder form successfully creates a ticket, the reminder is marked Completed and all reminder tokens are deleted.
- A failed ticket submission leaves the reminder Pending; scheduled emails continue according to its recurrence.
- The sender no longer completes a reminder merely because an unrelated ticket happens to match employee/category. Completion occurs through the secure reminder form.
