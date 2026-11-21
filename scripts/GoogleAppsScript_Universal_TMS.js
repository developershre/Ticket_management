/**
 * ============================================================================
 * UNIVERSAL GOOGLE APPS SCRIPT BACKEND FOR TICKET & TASK MANAGEMENT (TMS)
 * ============================================================================
 * 
 * This script provides a production-grade, scalable Global CRUD API for any
 * Google Sheet connected to the Ticket Management application.
 *
 * HOW TO INSTALL IN ANY GOOGLE SHEET:
 * 1. Open your Google Sheet.
 * 2. In the menu, go to: Extensions > Apps Script.
 * 3. Delete any default code in Code.gs, and paste the entirety of this file.
 * 4. Click "Save" (Ctrl+S or disk icon).
 * 5. Run the function `onOpen()` once from the toolbar to create the spreadsheet menu.
 * 6. Click "Deploy" > "New deployment".
 * 7. Choose type: "Web app".
 * 8. Description: "TMS Universal API".
 * 9. Execute as: "Me" (your email).
 * 10. Who has access: "Anyone".
 * 11. Click "Deploy", authorize permissions when prompted, and copy the Web App URL.
 * 12. Paste the Web App URL into the app's Settings > "API & Connection" tab.
 * ============================================================================
 */

// Configuration & Standard Schemas
const CONFIG = {
  VERSION: "2.0.0",
  DEFAULT_PAGE_SIZE: 50,
  MAX_PAGE_SIZE: 200,
  LOCK_TIMEOUT_MS: 30000,
  API_KEY: "", // Optional: leave blank for open/public REST, or set a string for legacy auth
};

const SCHEMAS = {
  Tickets: {
    sheetName: "Tickets",
    idColumn: "Ticket ID",
    idPrefix: "IT-",
    idPadding: 4,
    headers: [
      "Ticket ID",
      "Employee Code",
      "Email Address",
      "Category",
      "Subject",
      "Problem Description",
      "Attachment Name",
      "Attachment URL",
      "Priority",
      "Assigned to",
      "Customer",
      "Time Spent",
      "Activities",
      "My Activity",
      "Deadline",
      "SLA Status",
      "SLA Deadline",
      "Rating Avg Text",
      "Kanban State",
      "Stage",
      "Status",
      "Created on",
      "Last Updated on"
    ],
    defaults: {
      "Priority": "Medium",
      "Assigned to": "IT Support",
      "Customer": "Internal",
      "Time Spent": 0,
      "Kanban State": "Pending",
      "Stage": "Triage",
      "Status": "Open",
      "SLA Status": "Within SLA",
      "Rating Avg Text": "Unrated"
    }
  },
  Employees: {
    sheetName: "Employees",
    idColumn: "Employee Code",
    headers: [
      "Employee Code",
      "Name",
      "Dept",
      "Division",
      "Mail"
    ],
    defaults: {
      "Dept": "General",
      "Division": "Headquarters"
    }
  },
  Reminders: {
    sheetName: "Reminders",
    idColumn: "Reminder ID",
    idPrefix: "REM-",
    headers: [
      "Reminder ID",
      "Employee Code",
      "Employee Name",
      "Email Address",
      "Service Category",
      "Reminder Message",
      "Send At",
      "Status",
      "Email Sent At",
      "Ticket ID",
      "Created At",
      "Last Updated At",
      "CC Emails",
      "Repeat Frequency"
    ],
    defaults: {
      "Status": "Pending",
      "Email Sent At": "Not sent yet",
      "Repeat Frequency": "daily"
    }
  },
  Notes: {
    sheetName: "Notes",
    idColumn: "Note ID",
    idPrefix: "NOTE-",
    headers: [
      "Note ID",
      "Employee Code",
      "Content",
      "Created At",
      "Last Updated At",
      "Is Deleted"
    ],
    defaults: {
      "Is Deleted": 0
    }
  }
};

// Aliases for header resolution across various styles (snake_case, camelCase, Title Case)
const COLUMN_ALIASES = {
  // Tickets
  "ticket_id": "Ticket ID",
  "ticketid": "Ticket ID",
  "id": "Ticket ID",
  "employee_code": "Employee Code",
  "employeecode": "Employee Code",
  "emp_code": "Employee Code",
  "code": "Employee Code",
  "email_address": "Email Address",
  "email": "Email Address",
  "mail": "Email Address",
  "problem_description": "Problem Description",
  "description": "Problem Description",
  "problem": "Problem Description",
  "attachment_name": "Attachment Name",
  "attachmentname": "Attachment Name",
  "filename": "Attachment Name",
  "attachment_url": "Attachment URL",
  "attachmenturl": "Attachment URL",
  "fileurl": "Attachment URL",
  "assigned_to": "Assigned to",
  "assignedto": "Assigned to",
  "agent": "Assigned to",
  "assignee": "Assigned to",
  "time_spent": "Time Spent",
  "timespent": "Time Spent",
  "hours": "Time Spent",
  "activities": "Activities",
  "activities_text": "Activities",
  "my_activity": "My Activity",
  "myactivity": "My Activity",
  "sla_status": "SLA Status",
  "slastatus": "SLA Status",
  "sla_deadline": "SLA Deadline",
  "sladeadline": "SLA Deadline",
  "rating_avg_text": "Rating Avg Text",
  "rating": "Rating Avg Text",
  "kanban_state": "Kanban State",
  "kanbanstate": "Kanban State",
  "created_on": "Created on",
  "createdon": "Created on",
  "created_at": "Created on",
  "createdat": "Created on",
  "last_updated_on": "Last Updated on",
  "lastupdatedon": "Last Updated on",
  "last_updated_at": "Last Updated on",
  "lastupdatedat": "Last Updated on",
  "updated_at": "Last Updated on",

  // Employees
  "name": "Name",
  "emp_name": "Name",
  "dept": "Dept",
  "department": "Dept",
  "division": "Division",

  // Reminders
  "reminder_id": "Reminder ID",
  "reminderid": "Reminder ID",
  "employee_name": "Employee Name",
  "service_category": "Service Category",
  "reminder_message": "Reminder Message",
  "message": "Reminder Message",
  "send_at": "Send At",
  "email_sent_at": "Email Sent At",
  "cc_emails": "CC Emails",
  "repeat_frequency": "Repeat Frequency",

  // Notes
  "note_id": "Note ID",
  "noteid": "Note ID",
  "content": "Content",
  "is_deleted": "Is Deleted"
};

// ============================================================================
// Spreadsheet Custom Menu & Initialization
// ============================================================================

function onOpen() {
  SpreadsheetApp.getUi()
    .createMenu("TMS Support App")
    .addItem("Initialize / Verify All Sheets", "menuInitSchema")
    .addItem("Generate Sample Data", "menuSeedSampleData")
    .addToUi();
}

function menuInitSchema() {
  const result = initAllSheets();
  SpreadsheetApp.getUi().alert("TMS Initialization Result", JSON.stringify(result, null, 2), SpreadsheetApp.getUi().ButtonSet.OK);
}

function menuSeedSampleData() {
  initAllSheets();
  seedSampleData();
  SpreadsheetApp.getUi().alert("TMS Sample Data", "Sample Employees and Tickets have been generated successfully.", SpreadsheetApp.getUi().ButtonSet.OK);
}

// ============================================================================
// HTTP Handlers (doGet and doPost)
// ============================================================================

function doGet(e) {
  try {
    const params = (e && e.parameter) || {};
    const api = (params.api || "").trim().toLowerCase();
    const action = (params.action || "").trim().toLowerCase();

    // Health check
    if (api === "health" || api === "ping" || !api) {
      return jsonResponse({
        success: true,
        status: "success",
        healthy: true,
        version: CONFIG.VERSION,
        timestamp: new Date().toISOString()
      });
    }

    // Schema Verification & Initialization
    if (api === "schema") {
      if (action === "init") {
        const initReport = initAllSheets();
        return jsonResponse({ success: true, data: initReport });
      }
      const verifyReport = verifyAllSheets();
      return jsonResponse({ success: true, data: verifyReport });
    }

    // Universal CRUD routes
    if (api === "crud") {
      const sheetParam = params.sheet || "Tickets";
      const crudAction = action || "list";
      return handleUniversalCrudGet(sheetParam, crudAction, params);
    }

    // Entity-specific GET routes
    if (api === "tickets") {
      if (params.id) {
        return jsonResponse({ success: true, data: crudGetRecord("Tickets", params.id) });
      }
      return handleTicketsList(params);
    }

    if (api === "employees") {
      const code = params.code || params.id;
      if (code) {
        return jsonResponse({ success: true, data: crudGetRecord("Employees", code) });
      }
      return jsonResponse({ success: true, data: crudListRecords("Employees", params).items });
    }

    if (api === "reminders") {
      if (params.id) {
        return jsonResponse({ success: true, data: crudGetRecord("Reminders", params.id) });
      }
      return jsonResponse({ success: true, data: crudListRecords("Reminders", params).items });
    }

    if (api === "notes") {
      if (params.id) {
        return jsonResponse({ success: true, data: crudGetRecord("Notes", params.id) });
      }
      return jsonResponse({ success: true, data: crudListRecords("Notes", params).items });
    }

    if (api === "analytics") {
      return jsonResponse({ success: true, data: computeAnalytics() });
    }

    return jsonError("UNKNOWN_ROUTE", `Unknown API route: ${api}`, 404);
  } catch (err) {
    return jsonError("SERVER_ERROR", err.message || String(err), 500);
  }
}

function doPost(e) {
  const lock = LockService.getScriptLock();
  try {
    lock.waitLock(CONFIG.LOCK_TIMEOUT_MS);
  } catch (lockErr) {
    return jsonError("LOCK_TIMEOUT", "Google Sheet is busy. Please retry in a few moments.", 429, true);
  }

  try {
    const params = (e && e.parameter) || {};
    let body = {};
    if (e && e.postData && e.postData.contents) {
      try {
        body = JSON.parse(e.postData.contents);
      } catch (pErr) {
        body = {};
      }
    }

    // Legacy JSON-RPC envelope: { version: "v1", action: "...", auth: {...}, data: {...} }
    if (body.action) {
      return handleLegacyEnvelope(body);
    }

    const api = (params.api || "").trim();
    const action = (params.action || "").trim().toLowerCase();

    // Schema Actions
    if (api.toLowerCase() === "schema") {
      if (action === "init") {
        return jsonResponse({ success: true, data: initAllSheets() });
      }
      return jsonResponse({ success: true, data: verifyAllSheets() });
    }

    // Universal CRUD routes
    if (api.toLowerCase() === "crud") {
      const sheetParam = params.sheet || body.sheet || "Tickets";
      const crudAction = action || body.action || "create";
      const id = params.id || body.id;
      return handleUniversalCrudPost(sheetParam, crudAction, id, body);
    }

    // REST Ticket routes
    if (api.startsWith("tickets")) {
      return handleTicketRestPost(api, params, body);
    }

    // REST Employee routes
    if (api.startsWith("employees")) {
      return handleEmployeeRestPost(api, params, body);
    }

    // REST Reminder routes
    if (api.startsWith("reminders")) {
      return handleReminderRestPost(api, params, body);
    }

    // REST Notes routes
    if (api.startsWith("notes")) {
      return handleNotesRestPost(api, params, body);
    }

    return jsonError("UNKNOWN_ROUTE", `Unsupported POST API route: ${api}`, 404);
  } catch (err) {
    return jsonError("SERVER_ERROR", err.message || String(err), 500);
  } finally {
    try {
      lock.releaseLock();
    } catch (_) {}
  }
}

// ============================================================================
// Universal CRUD Engine
// ============================================================================

function handleUniversalCrudGet(sheetParam, action, params) {
  const schema = resolveSchema(sheetParam);
  if (!schema) {
    return jsonError("INVALID_SHEET", `Sheet '${sheetParam}' is not configured in TMS schema.`, 400);
  }

  if (action === "get") {
    const id = params.id;
    if (!id) return jsonError("VALIDATION_ERROR", "Missing required query parameter: id", 400);
    const item = crudGetRecord(schema.sheetName, id);
    if (!item) return jsonError("NOT_FOUND", `Record '${id}' not found in ${schema.sheetName}`, 404);
    return jsonResponse({ success: true, data: item });
  }

  // list
  const result = crudListRecords(schema.sheetName, params);
  return jsonResponse({ success: true, data: result });
}

function handleUniversalCrudPost(sheetParam, action, id, body) {
  const schema = resolveSchema(sheetParam);
  if (!schema) {
    return jsonError("INVALID_SHEET", `Sheet '${sheetParam}' is not configured in TMS schema.`, 400);
  }

  const payload = body.data || body.record || body;

  switch (action) {
    case "create": {
      const created = crudCreateRecord(schema.sheetName, payload);
      return jsonResponse({ success: true, data: created });
    }
    case "update": {
      const targetId = id || payload[schema.idColumn] || payload.id;
      if (!targetId) return jsonError("VALIDATION_ERROR", "Missing required ID for update", 400);
      const updated = crudUpdateRecord(schema.sheetName, targetId, payload);
      return jsonResponse({ success: true, data: updated });
    }
    case "delete": {
      const targetId = id || payload[schema.idColumn] || payload.id;
      if (!targetId) return jsonError("VALIDATION_ERROR", "Missing required ID for delete", 400);
      crudDeleteRecord(schema.sheetName, targetId);
      return jsonResponse({ success: true, message: `Record '${targetId}' deleted successfully.` });
    }
    default:
      return jsonError("INVALID_ACTION", `Unsupported CRUD action '${action}'`, 400);
  }
}

// ============================================================================
// Core Database Operations (Sheets API wrapper)
// ============================================================================

function getSpreadsheet() {
  return SpreadsheetApp.getActiveSpreadsheet();
}

function getOrInitSheet(sheetName) {
  const ss = getSpreadsheet();
  let sheet = ss.getSheetByName(sheetName);
  if (!sheet) {
    sheet = ss.insertSheet(sheetName);
    const schema = SCHEMAS[sheetName];
    if (schema) {
      sheet.getRange(1, 1, 1, schema.headers.length).setValues([schema.headers]);
      formatHeaderRow(sheet, schema.headers.length);
    }
  }
  return sheet;
}

function getHeaderMap(sheet) {
  const lastCol = sheet.getLastColumn();
  if (lastCol === 0) return { map: {}, list: [] };
  const rawHeaders = sheet.getRange(1, 1, 1, lastCol).getValues()[0];
  const map = {};
  const list = [];

  for (let i = 0; i < rawHeaders.length; i++) {
    const orig = String(rawHeaders[i] || "").trim();
    if (!orig) continue;
    list.push(orig);
    map[orig] = i + 1; // 1-indexed

    // Also index normalized variants
    const norm = normalizeKey(orig);
    map[norm] = i + 1;
  }
  return { map, list };
}

function normalizeKey(str) {
  return String(str || "")
    .toLowerCase()
    .replace(/[^a-z0-9]/g, "");
}

function resolveColumnIndex(headerMap, canonicalName) {
  if (headerMap.map[canonicalName]) return headerMap.map[canonicalName];
  const norm = normalizeKey(canonicalName);
  if (headerMap.map[norm]) return headerMap.map[norm];

  // Try aliases
  for (const alias in COLUMN_ALIASES) {
    if (COLUMN_ALIASES[alias] === canonicalName) {
      const aliasNorm = normalizeKey(alias);
      if (headerMap.map[aliasNorm]) return headerMap.map[aliasNorm];
    }
  }
  return null;
}

function rowToObject(rowValues, headerList) {
  const obj = {};
  for (let i = 0; i < headerList.length; i++) {
    const colName = headerList[i];
    let val = rowValues[i];
    if (val instanceof Date) {
      val = val.toISOString();
    } else if (val === null || val === undefined) {
      val = "";
    }
    obj[colName] = val;
  }
  return obj;
}

// ============================================================================
// CRUD Primitives
// ============================================================================

function crudListRecords(sheetName, params) {
  const sheet = getOrInitSheet(sheetName);
  const lastRow = sheet.getLastRow();
  const { map, list: headerList } = getHeaderMap(sheet);

  if (lastRow <= 1 || headerList.length === 0) {
    return { items: [], total: 0, offset: 0, limit: CONFIG.DEFAULT_PAGE_SIZE, has_more: false };
  }

  const offset = Math.max(0, parseInt(params.offset || 0, 10));
  const limit = Math.min(CONFIG.MAX_PAGE_SIZE, Math.max(1, parseInt(params.limit || CONFIG.DEFAULT_PAGE_SIZE, 10)));
  const query = (params.q || params.query || "").trim().toLowerCase();

  const numDataRows = lastRow - 1;
  const rawData = sheet.getRange(2, 1, numDataRows, sheet.getLastColumn()).getValues();

  let matched = [];
  for (let r = 0; r < rawData.length; r++) {
    const item = rowToObject(rawData[r], headerList);
    // Ignore soft-deleted rows if applicable
    if (item["Is Deleted"] === 1 || item["Is Deleted"] === "1" || item["Is Deleted"] === true) {
      continue;
    }

    if (query) {
      const matchFound = Object.values(item).some(v => String(v).toLowerCase().includes(query));
      if (!matchFound) continue;
    }
    matched.push(item);
  }

  const total = matched.length;
  const pagedItems = matched.slice(offset, offset + limit);
  const has_more = offset + limit < total;

  return {
    items: pagedItems,
    total,
    offset,
    limit,
    has_more
  };
}

function crudGetRecord(sheetName, idValue) {
  const schema = SCHEMAS[sheetName];
  if (!schema) return null;

  const sheet = getOrInitSheet(sheetName);
  const lastRow = sheet.getLastRow();
  if (lastRow <= 1) return null;

  const { map, list: headerList } = getHeaderMap(sheet);
  const idColIdx = resolveColumnIndex({ map, list: headerList }, schema.idColumn);
  if (!idColIdx) return null;

  const rawData = sheet.getRange(2, 1, lastRow - 1, sheet.getLastColumn()).getValues();
  const searchId = String(idValue).trim().toLowerCase();

  for (let r = 0; r < rawData.length; r++) {
    const val = String(rawData[r][idColIdx - 1] || "").trim().toLowerCase();
    if (val === searchId || val === searchId.replace(/^it-0*/, "") || searchId === `it-${val}`) {
      return rowToObject(rawData[r], headerList);
    }
  }
  return null;
}

function crudCreateRecord(sheetName, fields) {
  const schema = SCHEMAS[sheetName];
  if (!schema) throw new Error(`Unknown schema for sheet '${sheetName}'`);

  const sheet = getOrInitSheet(sheetName);
  const { map, list: headerList } = getHeaderMap(sheet);
  const nowStr = new Date().toISOString();

  // Merge defaults
  const record = Object.assign({}, schema.defaults || {}, fields);

  // Auto-generate ID if not provided
  const idCol = schema.idColumn;
  if (!record[idCol] && !record.id && !record[normalizeKey(idCol)]) {
    record[idCol] = generateNextId(sheet, schema);
  } else {
    record[idCol] = record[idCol] || record.id || record[normalizeKey(idCol)];
  }

  // Set timestamps
  if (headerList.includes("Created on")) record["Created on"] = nowStr;
  if (headerList.includes("Created At")) record["Created At"] = nowStr;
  if (headerList.includes("Last Updated on")) record["Last Updated on"] = nowStr;
  if (headerList.includes("Last Updated At")) record["Last Updated At"] = nowStr;

  // Build row array according to existing header order
  const newRow = [];
  for (let c = 0; c < headerList.length; c++) {
    const headerName = headerList[c];
    let value = findFieldInRecord(record, headerName);
    if (value === undefined || value === null) value = "";
    newRow.push(value);
  }

  sheet.appendRow(newRow);
  return crudGetRecord(sheetName, record[idCol]) || record;
}

function crudUpdateRecord(sheetName, idValue, patch) {
  const schema = SCHEMAS[sheetName];
  if (!schema) throw new Error(`Unknown schema for sheet '${sheetName}'`);

  const sheet = getOrInitSheet(sheetName);
  const lastRow = sheet.getLastRow();
  if (lastRow <= 1) throw new Error(`Record '${idValue}' not found in ${sheetName}`);

  const { map, list: headerList } = getHeaderMap(sheet);
  const idColIdx = resolveColumnIndex({ map, list: headerList }, schema.idColumn);
  if (!idColIdx) throw new Error(`Primary key column '${schema.idColumn}' missing from ${sheetName}`);

  const rawData = sheet.getRange(2, 1, lastRow - 1, sheet.getLastColumn()).getValues();
  const searchId = String(idValue).trim().toLowerCase();
  let targetRowIdx = -1;

  for (let r = 0; r < rawData.length; r++) {
    const val = String(rawData[r][idColIdx - 1] || "").trim().toLowerCase();
    if (val === searchId || val === searchId.replace(/^it-0*/, "") || searchId === `it-${val}`) {
      targetRowIdx = r + 2; // Sheet 1-indexed (row 1 is header)
      break;
    }
  }

  if (targetRowIdx === -1) {
    throw new Error(`Record '${idValue}' not found in ${sheetName}`);
  }

  const nowStr = new Date().toISOString();
  if (headerList.includes("Last Updated on") && patch["Last Updated on"] === undefined) {
    patch["Last Updated on"] = nowStr;
  }
  if (headerList.includes("Last Updated At") && patch["Last Updated At"] === undefined) {
    patch["Last Updated At"] = nowStr;
  }

  // Update specific cells
  for (let c = 0; c < headerList.length; c++) {
    const headerName = headerList[c];
    if (headerName === schema.idColumn) continue; // Do not overwrite primary key

    const newVal = findFieldInRecord(patch, headerName);
    if (newVal !== undefined) {
      sheet.getRange(targetRowIdx, c + 1).setValue(newVal);
    }
  }

  return crudGetRecord(sheetName, idValue);
}

function crudDeleteRecord(sheetName, idValue) {
  const schema = SCHEMAS[sheetName];
  if (!schema) throw new Error(`Unknown schema for sheet '${sheetName}'`);

  const sheet = getOrInitSheet(sheetName);
  const lastRow = sheet.getLastRow();
  if (lastRow <= 1) return true;

  const { map, list: headerList } = getHeaderMap(sheet);
  const idColIdx = resolveColumnIndex({ map, list: headerList }, schema.idColumn);
  if (!idColIdx) return true;

  const rawData = sheet.getRange(2, 1, lastRow - 1, sheet.getLastColumn()).getValues();
  const searchId = String(idValue).trim().toLowerCase();

  for (let r = 0; r < rawData.length; r++) {
    const val = String(rawData[r][idColIdx - 1] || "").trim().toLowerCase();
    if (val === searchId || val === searchId.replace(/^it-0*/, "") || searchId === `it-${val}`) {
      const sheetRow = r + 2;
      // If soft-delete column exists in Notes, mark it
      if (sheetName === "Notes" && headerList.includes("Is Deleted")) {
        const delColIdx = map["Is Deleted"];
        sheet.getRange(sheetRow, delColIdx).setValue(1);
        return true;
      }
      // If Tickets, can set Status = 'Cancelled'
      sheet.deleteRow(sheetRow);
      return true;
    }
  }
  return true;
}

// ============================================================================
// Specialized REST & RPC Handlers
// ============================================================================

function handleTicketsList(params) {
  const result = crudListRecords("Tickets", params);
  return jsonResponse({
    success: true,
    data: {
      tickets: result.items,
      items: result.items,
      total: result.total,
      offset: result.offset,
      limit: result.limit,
      has_more: result.has_more
    }
  });
}

function handleTicketRestPost(route, params, body) {
  const parts = route.split("/").filter(p => p.length > 0);
  // parts[0] is "tickets"

  // POST ?api=tickets/:id/stage
  if (parts.length >= 3 && parts[2] === "stage") {
    const ticketId = parts[1];
    const newStage = body.stage || body.kanban_state || "Pending";
    const updated = crudUpdateRecord("Tickets", ticketId, {
      "Stage": newStage,
      "Kanban State": newStage
    });
    return jsonResponse({ success: true, ticket: updated, data: updated });
  }

  // POST ?api=tickets/:id/close
  if (parts.length >= 3 && parts[2] === "close") {
    const ticketId = parts[1];
    const rating = body.rating || "5 Stars";
    const updated = crudUpdateRecord("Tickets", ticketId, {
      "Status": "Closed",
      "Stage": "Closed",
      "Kanban State": "Done",
      "Rating Avg Text": rating
    });
    return jsonResponse({ success: true, ticket: updated, data: updated });
  }

  // POST ?api=tickets/:id/delete
  if (parts.length >= 3 && parts[2] === "delete") {
    const ticketId = parts[1];
    crudDeleteRecord("Tickets", ticketId);
    return jsonResponse({ success: true, message: `Ticket ${ticketId} deleted.` });
  }

  // POST ?api=tickets&id=... (Update)
  const ticketId = params.id || (parts.length === 2 ? parts[1] : null);
  if (ticketId) {
    const updated = crudUpdateRecord("Tickets", ticketId, body);
    return jsonResponse({ success: true, ticket: updated, data: updated });
  }

  // POST ?api=tickets (Create)
  const created = crudCreateRecord("Tickets", body);
  return jsonResponse({ success: true, ticket: created, data: created });
}

function handleEmployeeRestPost(route, params, body) {
  const code = params.code || params.id;
  const isDelete = params.action === "delete" || body.action === "delete";

  if (isDelete && code) {
    crudDeleteRecord("Employees", code);
    return jsonResponse({ success: true, message: `Employee ${code} deleted.` });
  }

  if (code) {
    const updated = crudUpdateRecord("Employees", code, body);
    return jsonResponse({ success: true, employee: updated, data: updated });
  }

  const created = crudCreateRecord("Employees", body);
  return jsonResponse({ success: true, employee: created, data: created });
}

function handleReminderRestPost(route, params, body) {
  const id = params.id;
  const isSendNow = params.action === "send_now" || body.action === "send_now" || route.endsWith("/send_now");
  const isDelete = params.action === "delete" || body.action === "delete" || route.endsWith("/delete");

  if (isSendNow && id) {
    const updated = crudUpdateRecord("Reminders", id, {
      "Status": "Sent",
      "Email Sent At": new Date().toISOString()
    });
    return jsonResponse({ success: true, reminder: updated, data: updated });
  }

  if (isDelete && id) {
    crudDeleteRecord("Reminders", id);
    return jsonResponse({ success: true, message: `Reminder ${id} deleted.` });
  }

  if (id) {
    const updated = crudUpdateRecord("Reminders", id, body);
    return jsonResponse({ success: true, reminder: updated, data: updated });
  }

  const created = crudCreateRecord("Reminders", body);
  return jsonResponse({ success: true, reminder: created, data: created });
}

function handleNotesRestPost(route, params, body) {
  const id = params.id;
  const isDelete = params.action === "delete" || body.action === "delete" || route.endsWith("/delete");

  if (isDelete && id) {
    crudDeleteRecord("Notes", id);
    return jsonResponse({ success: true, message: `Note ${id} deleted.` });
  }

  if (id) {
    const updated = crudUpdateRecord("Notes", id, body);
    return jsonResponse({ success: true, note: updated, data: updated });
  }

  const created = crudCreateRecord("Notes", body);
  return jsonResponse({ success: true, note: created, data: created });
}

function handleLegacyEnvelope(body) {
  const action = body.action || "";
  const data = body.data || {};

  switch (action) {
    case "tickets.list": {
      const paged = crudListRecords("Tickets", data);
      return jsonResponse({ success: true, data: { tickets: paged.items, total: paged.total, has_more: paged.has_more } });
    }
    case "tickets.get": {
      const ticket = crudGetRecord("Tickets", data.ticket_id || data.id);
      if (!ticket) return jsonError("NOT_FOUND", "Ticket not found", 404);
      return jsonResponse({ success: true, data: { ticket } });
    }
    case "tickets.create": {
      const created = crudCreateRecord("Tickets", data);
      return jsonResponse({ success: true, data: { ticket: created } });
    }
    case "tickets.update": {
      const id = data.ticket_id || data.id;
      const updated = crudUpdateRecord("Tickets", id, data);
      return jsonResponse({ success: true, data: { ticket: updated } });
    }
    case "tickets.delete": {
      crudDeleteRecord("Tickets", data.ticket_id || data.id);
      return jsonResponse({ success: true, message: "Ticket deleted" });
    }
    case "employees.list": {
      const items = crudListRecords("Employees", data).items;
      return jsonResponse({ success: true, data: items });
    }
    case "reminders.list": {
      const items = crudListRecords("Reminders", data).items;
      return jsonResponse({ success: true, data: items });
    }
    case "schema.verify": {
      return jsonResponse({ success: true, data: verifyAllSheets() });
    }
    case "schema.init": {
      return jsonResponse({ success: true, data: initAllSheets() });
    }
    default:
      return jsonError("UNKNOWN_ACTION", `Legacy action '${action}' not supported.`, 400);
  }
}

// ============================================================================
// Schema Initialization & Verification
// ============================================================================

function initAllSheets() {
  const ss = getSpreadsheet();
  const report = {
    valid: true,
    spreadsheet_title: ss.getName(),
    sheets: {},
    created: []
  };

  for (const sheetKey in SCHEMAS) {
    const schema = SCHEMAS[sheetKey];
    let sheet = ss.getSheetByName(schema.sheetName);
    let wasCreated = false;

    if (!sheet) {
      sheet = ss.insertSheet(schema.sheetName);
      wasCreated = true;
      report.created.push(schema.sheetName);
    }

    const { map, list: existingHeaders } = getHeaderMap(sheet);
    if (existingHeaders.length === 0) {
      sheet.getRange(1, 1, 1, schema.headers.length).setValues([schema.headers]);
      formatHeaderRow(sheet, schema.headers.length);
    } else {
      // Append any missing standard headers to the end of the sheet
      const missing = [];
      for (const req of schema.headers) {
        if (!resolveColumnIndex({ map, list: existingHeaders }, req)) {
          missing.push(req);
        }
      }
      if (missing.length > 0) {
        sheet.getRange(1, existingHeaders.length + 1, 1, missing.length).setValues([missing]);
        formatHeaderRow(sheet, existingHeaders.length + missing.length);
      }
    }

    report.sheets[schema.sheetName] = {
      exists: true,
      headers_configured: schema.headers.length
    };
  }

  return report;
}

function verifyAllSheets() {
  const ss = getSpreadsheet();
  const report = {
    valid: true,
    spreadsheet_title: ss.getName(),
    sheets: {},
    missing_sheets: [],
    errors: []
  };

  for (const sheetKey in SCHEMAS) {
    const schema = SCHEMAS[sheetKey];
    const sheet = ss.getSheetByName(schema.sheetName);

    if (!sheet) {
      report.valid = false;
      report.missing_sheets.push(schema.sheetName);
      report.sheets[schema.sheetName] = {
        exists: false,
        total_rows: 0,
        columns_found: [],
        missing_columns: schema.headers,
        is_valid: false
      };
      continue;
    }

    const { map, list: foundList } = getHeaderMap(sheet);
    const missing = [];

    for (const req of schema.headers) {
      if (!resolveColumnIndex({ map, list: foundList }, req)) {
        missing.push(req);
      }
    }

    const isValid = missing.length === 0;
    if (!isValid) report.valid = false;

    report.sheets[schema.sheetName] = {
      exists: true,
      total_rows: Math.max(0, sheet.getLastRow() - 1),
      columns_found: foundList,
      missing_columns: missing,
      is_valid: isValid
    };
  }

  return report;
}

function formatHeaderRow(sheet, numCols) {
  const headerRange = sheet.getRange(1, 1, 1, numCols);
  headerRange.setFontWeight("bold");
  headerRange.setBackground("#2563eb");
  headerRange.setFontColor("#ffffff");
  headerRange.setHorizontalAlignment("center");
  sheet.setFrozenRows(1);
  for (let c = 1; c <= numCols; c++) {
    sheet.autoResizeColumn(c);
  }
}

// ============================================================================
// Helper Utilities
// ============================================================================

function resolveSchema(sheetParam) {
  const norm = sheetParam.trim().toLowerCase();
  for (const key in SCHEMAS) {
    if (key.toLowerCase() === norm || SCHEMAS[key].sheetName.toLowerCase() === norm) {
      return SCHEMAS[key];
    }
  }
  return null;
}

function findFieldInRecord(record, headerName) {
  if (record[headerName] !== undefined) return record[headerName];

  // Try direct normalized key
  const norm = normalizeKey(headerName);
  if (record[norm] !== undefined) return record[norm];

  // Try known aliases
  for (const alias in COLUMN_ALIASES) {
    if (COLUMN_ALIASES[alias] === headerName) {
      if (record[alias] !== undefined) return record[alias];
      const aliasNorm = normalizeKey(alias);
      if (record[aliasNorm] !== undefined) return record[aliasNorm];
    }
  }
  return undefined;
}

function generateNextId(sheet, schema) {
  const lastRow = sheet.getLastRow();
  const prefix = schema.idPrefix || "ID-";
  const padding = schema.idPadding || 4;

  if (lastRow <= 1) {
    return `${prefix}${String(1).padStart(padding, "0")}`;
  }

  const { map, list: headerList } = getHeaderMap(sheet);
  const idColIdx = resolveColumnIndex({ map, list: headerList }, schema.idColumn);
  if (!idColIdx) {
    return `${prefix}${String(lastRow).padStart(padding, "0")}`;
  }

  const values = sheet.getRange(2, idColIdx, lastRow - 1, 1).getValues();
  let maxNum = 0;

  for (let i = 0; i < values.length; i++) {
    const raw = String(values[i][0] || "").trim();
    const numPart = raw.replace(/\D/g, "");
    if (numPart) {
      const n = parseInt(numPart, 10);
      if (!isNaN(n) && n > maxNum) maxNum = n;
    }
  }

  const nextNum = maxNum + 1;
  return `${prefix}${String(nextNum).padStart(padding, "0")}`;
}

function computeAnalytics() {
  const ticketList = crudListRecords("Tickets", { limit: 1000 }).items;
  const stages = {};
  const priorities = {};
  const categories = {};

  ticketList.forEach(t => {
    const st = t.Stage || t["Kanban State"] || "Triage";
    stages[st] = (stages[st] || 0) + 1;

    const pr = t.Priority || "Medium";
    priorities[pr] = (priorities[pr] || 0) + 1;

    const cat = t.Category || "General";
    categories[cat] = (categories[cat] || 0) + 1;
  });

  return {
    total_tickets: ticketList.length,
    by_stage: stages,
    by_priority: priorities,
    by_category: categories
  };
}

function seedSampleData() {
  // Seed sample employees if empty
  const empSheet = getOrInitSheet("Employees");
  if (empSheet.getLastRow() <= 1) {
    const sampleEmployees = [
      ["11061", "Chaitram Sahu", "IT Support", "Raipur", "chaitram@bpsl.com"],
      ["11062", "Devendra Patel", "Network Operations", "Raipur", "devendra@bpsl.com"],
      ["11063", "Amit Sharma", "Security & Systems", "Delhi", "amit.sharma@bpsl.com"],
      ["11064", "Pooja Verma", "Human Resources", "Mumbai", "pooja.verma@bpsl.com"]
    ];
    sampleEmployees.forEach(row => empSheet.appendRow(row));
  }

  // Seed sample tickets if empty
  const ticketSheet = getOrInitSheet("Tickets");
  if (ticketSheet.getLastRow() <= 1) {
    const nowStr = new Date().toISOString();
    const sampleTickets = [
      ["IT-0001", "11061", "chaitram@bpsl.com", "Hardware", "Workstation power supply failure", "Desktop CPU not powering on after power surge.", "", "", "High", "Chaitram Sahu", "Internal", 1.5, "Diagnosed faulty PSU", "Replacing power supply", "Tomorrow · 5:00 PM", "Within SLA", "Within 24 Hours (P2)", "Unrated", "InProgress", "In Progress", "Open", nowStr, nowStr],
      ["IT-0002", "11062", "devendra@bpsl.com", "Network", "Slow internet connection in Block B", "Employees reporting intermittent Wi-Fi disconnection.", "", "", "Medium", "Devendra Patel", "Internal", 0.5, "Rebooted access point AP-04", "Testing signal coverage", "Within 3 Business Days", "Within SLA", "Within 48 Hours (P3)", "Unrated", "Pending", "Triage", "Open", nowStr, nowStr]
    ];
    sampleTickets.forEach(row => ticketSheet.appendRow(row));
  }
}

function jsonResponse(data) {
  return ContentService.createTextOutput(JSON.stringify(data))
    .setMimeType(ContentService.MimeType.JSON);
}

function jsonError(code, message, status, retryable) {
  return ContentService.createTextOutput(JSON.stringify({
    success: false,
    status: "error",
    error: {
      code: code || "ERROR",
      message: message || "Unknown error occurred",
      retryable: !!retryable
    },
    message: message || "Unknown error"
  })).setMimeType(ContentService.MimeType.JSON);
}

