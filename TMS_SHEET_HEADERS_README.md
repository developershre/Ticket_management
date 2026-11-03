# TMS Google Sheets – Sheet Names and Headers

This document lists the worksheet tabs and exact header rows defined by the current TMS project (`MAIN.gs`, version 2.4.0).

> **Important:** Keep the sheet tab names and header labels exactly as shown below. Header order matters for the `Tickets` and `Reminders` sheets in the current implementation. The `Employee` API supports recognized header aliases and reordered columns, but the standard layout below is recommended.

## 1. Tickets

**Sheet tab name:** `Tickets`  
**Total columns:** 23  
**Header row (Row 1):**

| Column | Exact header | API field / purpose |
|---:|---|---|
| A | `Ticket ID` | Unique ticket identifier |
| B | `Employee Code` | Employee who raised the ticket |
| C | `Email Address` | Employee email address |
| D | `Category` | Ticket category |
| E | `Subject` | Short issue title |
| F | `Problem Description` | Full issue description |
| G | `Attachment Name` | Attachment file name(s), if any |
| H | `Attachment URL` | Google Drive attachment link(s), if any |
| I | `Priority` | Low, Normal, High, or Critical |
| J | `Assigned to` | Person/team assigned to the ticket |
| K | `Customer` | Employee/customer name; ticket form fills this from `Employee` → `Name` |
| L | `Time Spent` | Time spent resolving the ticket |
| M | `Activities` | Activity history / notes |
| N | `My Activity` | Current user's activity note |
| O | `Deadline` | Ticket deadline |
| P | `SLA Status` | SLA state |
| Q | `SLA Deadline` | SLA due date/time |
| R | `Rating Avg Text` | Rating summary/text |
| S | `Kanban State` | Kanban board state |
| T | `Stage` | Workflow stage |
| U | `Status` | Ticket status |
| V | `Created on` | Ticket creation timestamp |
| W | `Last Updated on` | Last modification timestamp |

**Copy/paste header row (tab-separated):**

```text
Ticket ID\tEmployee Code\tEmail Address\tCategory\tSubject\tProblem Description\tAttachment Name\tAttachment URL\tPriority\tAssigned to\tCustomer\tTime Spent\tActivities\tMy Activity\tDeadline\tSLA Status\tSLA Deadline\tRating Avg Text\tKanban State\tStage\tStatus\tCreated on\tLast Updated on
```

## 2. Employee

**Sheet tab name:** `Employee`  
**Standard columns:** 5  
**Header row (Row 1):**

| Column | Exact header | API field / purpose |
|---:|---|---|
| A | `Employee Code` | Unique employee code; required for employee lookup |
| B | `Name` | Employee's full name; used as the ticket `Customer` value |
| C | `Dept` | Department name |
| D | `Mail` | Employee email address |
| E | `Division` | Employee's division; used to filter employee selection in Create Reminder |

**Copy/paste header row (tab-separated):**

```text
Employee Code\tName\tDept\tMail\tDivision
```

**Employee header compatibility:** The employee API can recognize common alternatives such as `Employee ID` / `Emp Code`, `Employee Name` / `Full Name`, `Department`, `Email` / `Email Address`, and `Business Division`. For best compatibility across the forms and dashboard, use the exact standard headers above.

## 3. Reminders

**Sheet tab name:** `Reminders`  
**Total columns:** 13  
**Header row (Row 1):**

| Column | Exact header | API field / purpose |
|---:|---|---|
| A | `Reminder ID` | Unique reminder identifier |
| B | `Employee Code` | Employee receiving the reminder |
| C | `Employee Name` | Employee name copied from `Employee` sheet |
| D | `Email Address` | Recipient email address |
| E | `Service / Category` | Service or issue category |
| F | `Reminder Message` | Reminder message/details |
| G | `Send At` | Scheduled date/time for sending |
| H | `Status` | Reminder status, e.g. Pending or Completed |
| I | `Email Sent At` | Timestamp of the last successful email send |
| J | `Ticket ID` | Ticket created against this reminder, when completed |
| K | `Created At` | Reminder creation timestamp |
| L | `Last Updated At` | Last reminder update timestamp |
| M | `CC Emails` | Optional CC recipients; multiple addresses may be stored comma-separated |

**Copy/paste header row (tab-separated):**

```text
Reminder ID\tEmployee Code\tEmployee Name\tEmail Address\tService / Category\tReminder Message\tSend At\tStatus\tEmail Sent At\tTicket ID\tCreated At\tLast Updated At\tCC Emails
```

## Workbook summary

| Sheet tab | Required headers | Notes |
|---|---:|---|
| `Tickets` | 23 | Main ticket records |
| `Employee` | 5 | Employee master data; standard order shown above |
| `Reminders` | 13 | Scheduled reminders and optional CC recipients |

## Notes

- The current project defines **three Google Sheets tabs**: `Tickets`, `Employee`, and `Reminders`.
- Ticket categories and priority options are configured in the TMS script configuration (`MAIN.gs` / script properties), not in a separate worksheet in this project version.
- Do not rename the sheet tabs unless you also update the corresponding `TMS.SHEETS` values in `MAIN.gs`.
- Do not remove the `CC Emails` column if using reminder CC functionality.
- If an older `Reminders` sheet has only 12 columns, add `CC Emails` as column M (13th column) or run the project's sheet initialization/migration function before using CC support.
