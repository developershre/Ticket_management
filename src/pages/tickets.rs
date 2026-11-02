//! IT Ticket Management — Kanban Board (Pending, InProgress, Onhold, Completed)
//! and full-width interactive DataTable with drag & drop support.
#![allow(clippy::redundant_closure)]

use dioxus::prelude::*;

use crate::components::ui::badge::Badge;
use crate::components::ui::button::Button;
use crate::components::ui::context_menu::{
    ContextMenu, ContextMenuContent, ContextMenuItem, ContextMenuLabel, ContextMenuSeparator,
    ContextMenuSub, ContextMenuSubContent, ContextMenuSubTrigger, ContextMenuTrigger,
};
use crate::components::ui::data_table::{DataTable, DataTableColumn, DataTableFilter, SortValue};
use crate::components::ui::dialog::{
    Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
};
use crate::components::ui::dropdown_menu::{
    DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuLabel, DropdownMenuSeparator,
    DropdownMenuTrigger,
};
use crate::components::ui::input::Input;
use crate::components::ui::tabs::{Tabs, TabsContent, TabsList, TabsTrigger};
use crate::components::ui::toast::ToastState;
use crate::icons::{
    CheckIcon, CopyIcon, EyeIcon, FilterIcon, GripVerticalIcon, MoreHorizontalIcon, PencilIcon,
    PlusIcon, SearchIcon, TrashIcon,
};

/// IT Ticket Item model storing all 23 standardized fields.
#[derive(Clone, PartialEq, Debug)]
pub struct TicketItem {
    // 1. Ticket ID
    pub ticket_id: i64,
    // 2. Employee Code
    pub employee_code: String,
    // 3. Email Address
    pub email_address: String,
    // 4. Category
    pub category: String,
    // 5. Subject
    pub subject: String,
    // 6. Problem Description
    pub problem_description: String,
    // 7. Attachment Name
    pub attachment_name: String,
    // 8. Attachment URL
    pub attachment_url: String,
    // 9. Priority
    pub priority: String,
    // 10. Assigned to
    pub assigned_to: String,
    // 11. Customer
    pub customer: String,
    // 12. Time Spent (hours)
    pub time_spent: f64,
    // 13. Activities
    pub activities: String,
    // 14. My Activity
    pub my_activity: String,
    // 15. Deadline
    pub deadline: String,
    // 16. SLA Status
    pub sla_status: String,
    // 17. SLA Deadline
    pub sla_deadline: String,
    // 18. Rating Avg Text
    pub rating_avg_text: String,
    // 19. Kanban State ("Pending" | "InProgress" | "Onhold" | "Completed")
    pub kanban_state: String,
    // 20. Stage
    pub stage: String,
    // 21. Status
    pub status: String,
    // 22. Created on
    pub created_on: String,
    // 23. Last Updated on
    pub last_updated_on: String,

    // Aliases for convenience & UI compatibility:
    pub id: String,
    pub title: String,
    pub employee: String,
    pub assignee: String,
    pub created_at: String,
}

impl TicketItem {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        ticket_id: i64,
        employee_code: &str,
        email_address: &str,
        category: &str,
        subject: &str,
        problem_description: &str,
        attachment_name: &str,
        attachment_url: &str,
        priority: &str,
        assigned_to: &str,
        customer: &str,
        time_spent: f64,
        activities: &str,
        my_activity: &str,
        deadline: &str,
        sla_status: &str,
        sla_deadline: &str,
        rating_avg_text: &str,
        kanban_state: &str,
        stage: &str,
        status: &str,
        created_on: &str,
        last_updated_on: &str,
    ) -> Self {
        let id = format!("IT-{}", ticket_id);
        Self {
            ticket_id,
            employee_code: employee_code.to_string(),
            email_address: email_address.to_string(),
            category: category.to_string(),
            subject: subject.to_string(),
            problem_description: problem_description.to_string(),
            attachment_name: attachment_name.to_string(),
            attachment_url: attachment_url.to_string(),
            priority: priority.to_string(),
            assigned_to: assigned_to.to_string(),
            customer: customer.to_string(),
            time_spent,
            activities: activities.to_string(),
            my_activity: my_activity.to_string(),
            deadline: deadline.to_string(),
            sla_status: sla_status.to_string(),
            sla_deadline: sla_deadline.to_string(),
            rating_avg_text: rating_avg_text.to_string(),
            kanban_state: kanban_state.to_string(),
            stage: stage.to_string(),
            status: status.to_string(),
            created_on: created_on.to_string(),
            last_updated_on: last_updated_on.to_string(),

            // aliases:
            id,
            title: subject.to_string(),
            employee: employee_code.to_string(),
            assignee: assigned_to.to_string(),
            created_at: created_on.to_string(),
        }
    }

    pub fn from_db_ticket(t: crate::db::DbTicket) -> Self {
        let stage = match t.kanban_state.as_str() {
            "Pending" | "InProgress" | "Onhold" | "Completed" => t.kanban_state.clone(),
            _ => match t.stage.as_str() {
                "Pending" | "InProgress" | "Onhold" | "Completed" => t.stage.clone(),
                _ => match t.status.as_str() {
                    "InProgress" | "In Progress" => "InProgress".to_string(),
                    "Onhold" | "On Hold" => "Onhold".to_string(),
                    "Completed" | "Resolved" | "Closed" => "Completed".to_string(),
                    _ => "Pending".to_string(),
                },
            },
        };

        Self::new(
            t.ticket_id,
            &t.employee_code,
            &t.email_address,
            &t.category,
            &t.subject,
            &t.problem_description,
            &t.attachment_name,
            &t.attachment_url,
            &t.priority,
            &t.assigned_to,
            &t.customer,
            t.time_spent,
            &t.activities,
            &t.my_activity,
            &t.deadline,
            &t.sla_status,
            &t.sla_deadline,
            &t.rating_avg_text,
            &stage,
            &stage,
            &stage,
            &t.created_on,
            &t.last_updated_on,
        )
    }

    pub fn emp_code_only(&self) -> &str {
        if let Some(idx) = self.employee_code.find('(') {
            self.employee_code[..idx].trim()
        } else {
            self.employee_code.trim()
        }
    }

    pub fn emp_name(&self) -> String {
        if let (Some(start), Some(end)) = (self.employee_code.find('('), self.employee_code.rfind(')')) {
            if end > start {
                let inside = self.employee_code[start + 1..end].trim();
                if !inside.is_empty() {
                    return inside.to_string();
                }
            }
        }
        if let Some(user_part) = self.email_address.split('@').next() {
            let parts: Vec<String> = user_part
                .split('.')
                .map(|p| {
                    let mut c = p.chars();
                    match c.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                    }
                })
                .collect();
            if !parts.is_empty() {
                return parts.join(" ");
            }
        }
        self.employee_code.clone()
    }

    pub fn assignee_name(&self) -> &str {
        match self.assigned_to.as_str() {
            "GR" => "Gabriel Ross (Lead)",
            "AL" => "Alice Liu (Hardware)",
            "JD" => "John Doe (Access)",
            "SL" => "Sarah Lin (DevOps)",
            other if !other.is_empty() => other,
            _ => "Unassigned",
        }
    }
}

/// Global reactive state holder for viewing and modifying tickets
#[derive(Clone, Copy, PartialEq)]
pub struct TicketContextState {
    pub tickets: Signal<Vec<TicketItem>>,
    pub viewing_ticket: Signal<Option<TicketItem>>,
    pub editing_ticket: Signal<Option<TicketItem>>,
    pub is_view_open: Signal<bool>,
    pub is_edit_open: Signal<bool>,

    pub edit_emp_name: Signal<String>,
    pub edit_employee_code: Signal<String>,
    pub edit_email_address: Signal<String>,
    pub edit_category: Signal<String>,
    pub edit_subject: Signal<String>,
    pub edit_problem_description: Signal<String>,
    pub edit_attachment_name: Signal<String>,
    pub edit_attachment_url: Signal<String>,
    pub edit_priority: Signal<String>,
    pub edit_assigned_to: Signal<String>,
    pub edit_customer: Signal<String>,
    pub edit_time_spent: Signal<f64>,
    pub edit_activities: Signal<String>,
    pub edit_my_activity: Signal<String>,
    pub edit_deadline: Signal<String>,
    pub edit_sla_status: Signal<String>,
    pub edit_sla_deadline: Signal<String>,
    pub edit_rating_avg_text: Signal<String>,
    pub edit_kanban_state: Signal<String>,
    pub edit_stage: Signal<String>,
    pub edit_status: Signal<String>,
}

impl TicketContextState {
    pub fn open_view(&mut self, t: TicketItem) {
        self.viewing_ticket.set(Some(t));
        self.is_view_open.set(true);
    }

    pub fn open_edit(&mut self, t: TicketItem) {
        self.edit_emp_name.set(t.emp_name());
        self.edit_employee_code.set(t.emp_code_only().to_string());
        self.edit_email_address.set(t.email_address.clone());
        self.edit_category.set(t.category.clone());
        self.edit_subject.set(t.subject.clone());
        self.edit_problem_description.set(t.problem_description.clone());
        self.edit_attachment_name.set(t.attachment_name.clone());
        self.edit_attachment_url.set(t.attachment_url.clone());
        self.edit_priority.set(t.priority.clone());
        self.edit_assigned_to.set(t.assigned_to.clone());
        self.edit_customer.set(t.customer.clone());
        self.edit_time_spent.set(t.time_spent);
        self.edit_activities.set(t.activities.clone());
        self.edit_my_activity.set(t.my_activity.clone());
        self.edit_deadline.set(t.deadline.clone());
        self.edit_sla_status.set(t.sla_status.clone());
        self.edit_sla_deadline.set(t.sla_deadline.clone());
        self.edit_rating_avg_text.set(t.rating_avg_text.clone());
        self.edit_kanban_state.set(t.kanban_state.clone());
        self.edit_stage.set(t.kanban_state.clone());
        self.edit_status.set(t.kanban_state.clone());

        self.editing_ticket.set(Some(t));
        self.is_edit_open.set(true);
        self.is_view_open.set(false);
    }

    pub fn update_status(&mut self, ticket_id: &str, new_status: &str) {
        let mut list = self.tickets.write();
        if let Some(pos) = list.iter().position(|t| t.id == ticket_id) {
            list[pos].status = new_status.to_string();
            list[pos].kanban_state = new_status.to_string();
            list[pos].stage = new_status.to_string();
        }
        if let Some(num) = ticket_id.strip_prefix("IT-").and_then(|s| s.parse::<i64>().ok()) {
            let _ = crate::db::update_ticket_status(num, new_status);
        }
    }

    pub fn update_priority(&mut self, ticket_id: &str, new_priority: &str) {
        let mut list = self.tickets.write();
        if let Some(pos) = list.iter().position(|t| t.id == ticket_id) {
            list[pos].priority = new_priority.to_string();
        }
        if let Some(num) = ticket_id.strip_prefix("IT-").and_then(|s| s.parse::<i64>().ok()) {
            let _ = crate::db::update_ticket_priority(num, new_priority);
        }
    }

    pub fn delete_ticket(&mut self, ticket_id: &str) {
        let mut list = self.tickets.write();
        list.retain(|t| t.id != ticket_id);
        if let Some(num) = ticket_id.strip_prefix("IT-").and_then(|s| s.parse::<i64>().ok()) {
            let _ = crate::db::delete_ticket(num);
        }
    }
}

pub const COLUMN_NAMES: &[&str] = &["Pending", "InProgress", "Onhold", "Completed"];

fn initial_tickets() -> Vec<TicketItem> {
    let db_tickets = crate::db::get_all_tickets();
    if !db_tickets.is_empty() {
        return db_tickets
            .into_iter()
            .map(TicketItem::from_db_ticket)
            .collect();
    }

    vec![
        TicketItem::new(
            101,
            "EMP-4019 (David Chen)",
            "david.chen@corp.internal",
            "Network",
            "VPN connection timeout on Cisco AnyConnect",
            "Cisco AnyConnect fails to establish DTLS handshake when roaming from office Wi-Fi to cellular hotspot. Reinstalling certificates did not resolve.",
            "anyconnect_debug.log",
            "/attachments/anyconnect_debug.log",
            "High",
            "GR",
            "Internal",
            1.5,
            "Diagnostic logs requested; investigated MTU fragmentation over gateway tunnel.",
            "Reconfigured AnyConnect client profile MTU to 1350 and verified TLS handshake.",
            "2026-11-02 17:00",
            "Within SLA",
            "2026-11-01 12:00",
            "4.9 / 5.0",
            "InProgress",
            "Investigation",
            "In Progress",
            "Today · 09:15 AM",
            "Today · 10:45 AM",
        ),
        TicketItem::new(
            102,
            "EMP-5082 (Sarah Miller)",
            "sarah.miller@corp.internal",
            "Hardware",
            "MacBook Pro M3 workstation setup & provisioning",
            "New hire joining Frontend team on Monday. Requesting standard macOS engineering profile with Homebrew, Docker, Rust, and Node.js pre-imaged.",
            "workstation_checklist.pdf",
            "/attachments/workstation_checklist.pdf",
            "Medium",
            "AL",
            "Internal",
            2.0,
            "Hardware unboxed, serial logged into asset tracker; MDM enrollment profile assigned.",
            "Automated setup script executed; awaiting user pickup.",
            "2026-11-03 14:00",
            "Within SLA",
            "2026-11-02 18:00",
            "5.0 / 5.0",
            "Pending",
            "Provisioning",
            "Open",
            "Today · 10:30 AM",
            "Today · 11:00 AM",
        ),
        TicketItem::new(
            103,
            "EMP-1024 (Robert Taylor)",
            "robert.taylor@corp.internal",
            "Access",
            "Active Directory account lock after password expiry",
            "User locked out of corporate workstation and Okta SSO after 90-day password rotation cycle expired over the weekend.",
            "",
            "",
            "High",
            "JD",
            "Finance",
            0.5,
            "Verified employee identity via manager voice verification; reset AD badPwdCount.",
            "Temporary one-time password generated and communicated securely.",
            "Today · 12:00 PM",
            "Approaching Breach",
            "Today · 11:30 AM",
            "4.5 / 5.0",
            "Pending",
            "Triage",
            "Open",
            "Today · 11:00 AM",
            "Today · 11:15 AM",
        ),
        TicketItem::new(
            104,
            "EMP-3011 (Elena Rostova)",
            "elena.rostova@corp.internal",
            "Security",
            "SSL certificate expiring on internal staging gateway",
            "Wildcard TLS certificate for *.staging.internal expires in 48 hours. Reverse proxy ingress controller throwing expiration warnings.",
            "cert_status.json",
            "/attachments/cert_status.json",
            "High",
            "SL",
            "DevOps",
            3.0,
            "Generated CSR via internal Vault CA; cert issued and injected into Kubernetes TLS secret.",
            "Ingress controller reloaded; HTTPS handshake validated across staging microservices.",
            "Tomorrow · 09:00 AM",
            "Within SLA",
            "Tomorrow · 12:00 PM",
            "4.8 / 5.0",
            "InProgress",
            "Resolution",
            "In Progress",
            "Yesterday · 02:20 PM",
            "Today · 08:30 AM",
        ),
        TicketItem::new(
            105,
            "EMP-2201 (Mark Vance)",
            "mark.vance@corp.internal",
            "Hardware",
            "Replacement battery requested for Dell Latitude",
            "Dell Latitude 7420 battery health degraded to 42% capacity. System throttles CPU under moderate workloads.",
            "battery_report.html",
            "/attachments/battery_report.html",
            "Low",
            "GR",
            "Marketing",
            0.2,
            "Dell Premier support ticket #98231 logged for onsite OEM battery replacement.",
            "Vendor confirmed technician dispatch scheduled for Thursday morning.",
            "2026-11-06 18:00",
            "Within SLA",
            "2026-11-05 18:00",
            "4.7 / 5.0",
            "Onhold",
            "Awaiting Vendor",
            "Pending Vendor",
            "2 days ago",
            "Yesterday · 11:00 AM",
        ),
        TicketItem::new(
            106,
            "EMP-4109 (Lisa Wong)",
            "lisa.wong@corp.internal",
            "Access",
            "Awaiting manager approval for Production AWS IAM role",
            "Requested temporary AssumeRole permissions for AWS production ECS cluster debugging during scheduled maintenance window.",
            "",
            "",
            "Medium",
            "JD",
            "Cloud Ops",
            0.3,
            "Permission boundary check passed; automated approval notification routed to VP Engineering.",
            "Awaiting electronic signature on security sign-off form.",
            "2026-11-04 17:00",
            "Within SLA",
            "2026-11-04 12:00",
            "4.6 / 5.0",
            "Onhold",
            "Approval Pending",
            "Pending Approval",
            "3 days ago",
            "2 days ago",
        ),
        TicketItem::new(
            107,
            "EMP-1190 (James Smith)",
            "james.smith@corp.internal",
            "Hardware",
            "Floor 3 Kyocera network printer offline",
            "Office multifunction printer not responding to ping or SNMP requests; paper jam cleared but print queue hung.",
            "printer_snmp.log",
            "/attachments/printer_snmp.log",
            "Medium",
            "AL",
            "Operations",
            1.2,
            "Power cycled network card; cleared corrupted print spool on print server 01.",
            "Self-test page printed successfully; network queue verified clear.",
            "Oct 28 · 05:00 PM",
            "Within SLA",
            "Oct 28 · 06:00 PM",
            "5.0 / 5.0",
            "Completed",
            "Resolved",
            "Resolved",
            "Oct 28 · 04:40 PM",
            "Oct 28 · 05:25 PM",
        ),
        TicketItem::new(
            108,
            "EMP-3402 (Alex Kumar)",
            "alex.kumar@corp.internal",
            "Software",
            "Docker Engine fails to start on Ubuntu 24.04 dev VM",
            "systemd docker.service fails on restart with iptables backend error after kernel upgrade.",
            "dockerd_journal.txt",
            "/attachments/dockerd_journal.txt",
            "Medium",
            "SL",
            "Engineering",
            2.5,
            "Identified nftables/iptables-legacy conflict in /etc/docker/daemon.json.",
            "Switched iptables alternative to legacy mode; dockerd restarted cleanly.",
            "Oct 30 · 18:00",
            "Within SLA",
            "Oct 30 · 12:00",
            "4.9 / 5.0",
            "InProgress",
            "Investigation",
            "In Progress",
            "Oct 29 · 11:15 AM",
            "Today · 09:00 AM",
        ),
        TicketItem::new(
            109,
            "EMP-8021 (Finance Team)",
            "finance-desk@corp.internal",
            "Network",
            "Wi-Fi access point degraded in Conference Room C",
            "Excessive packet loss and low throughput reported during department quarterly town hall video conferences.",
            "wifi_heat_map.png",
            "/attachments/wifi_heat_map.png",
            "High",
            "GR",
            "Finance",
            0.8,
            "Cisco Catalyst 9120 AP RF profile analyzed; channel 36 experiencing high co-channel interference.",
            "Adjusted RRM channel assignment to DFS channel 104 and increased transmit power by 3dB.",
            "Today · 14:00",
            "Within SLA",
            "Today · 12:00",
            "4.7 / 5.0",
            "Pending",
            "Triage",
            "Open",
            "Today · 08:45 AM",
            "Today · 09:30 AM",
        ),
        TicketItem::new(
            110,
            "EMP-9012 (DevOps Team)",
            "devops@corp.internal",
            "Infrastructure",
            "Upgrade PostgreSQL replica memory allocation",
            "PostgreSQL 16 replica memory utilization exceeded 92% during peak read query load.",
            "",
            "",
            "High",
            "SL",
            "Engineering",
            4.0,
            "Staged maintenance window; scaled AWS RDS instance from db.r6g.xlarge to db.r6g.2xlarge.",
            "Replication lag dropped from 14s to <4ms; query latency normalized.",
            "Oct 27 · 06:00 PM",
            "Within SLA",
            "Oct 27 · 08:00 PM",
            "5.0 / 5.0",
            "Completed",
            "Resolved",
            "Resolved",
            "Oct 27 · 06:00 PM",
            "Oct 27 · 07:15 PM",
        ),
        TicketItem::new(
            111,
            "EMP-1000 (SecOps)",
            "secops@corp.internal",
            "Security",
            "Security patch deployment for Windows 11 fleet",
            "Cumulative quality and security updates (KB5036893) pending installation across 180 corporate workstations.",
            "fleet_patch_report.csv",
            "/attachments/fleet_patch_report.csv",
            "Medium",
            "JD",
            "SecOps",
            3.5,
            "Approved in WSUS pilot ring; phased roll-out scheduled across engineering and finance rings.",
            "Compliance reached 98.4%; remainder scheduled for automatic reboot at 03:00 AM.",
            "Oct 26 · 18:00",
            "Within SLA",
            "Oct 26 · 23:59",
            "4.9 / 5.0",
            "Completed",
            "Resolved",
            "Resolved",
            "Oct 26 · 02:00 PM",
            "Oct 26 · 05:30 PM",
        ),
        TicketItem::new(
            112,
            "EMP-6120 (Emily Davis)",
            "emily.davis@corp.internal",
            "Hardware",
            "External monitor flicker over DisplayPort adapter",
            "Dell UltraSharp U2723QE flickers intermittently when waking from sleep via Thunderbolt dock.",
            "",
            "",
            "Low",
            "AL",
            "Design",
            0.5,
            "Tested with alternate certified DP 1.4 cable; issue persists on docking station TB3 port.",
            "Awaiting replacement CalDigit TS4 docking station under warranty RMA.",
            "2026-11-05 17:00",
            "Within SLA",
            "2026-11-05 18:00",
            "4.6 / 5.0",
            "Onhold",
            "Awaiting RMA",
            "Pending Vendor",
            "Yesterday · 04:00 PM",
            "Today · 10:15 AM",
        ),
    ]
}

fn column_display_label(col: &str) -> &'static str {
    match col {
        "Pending" => "Pending",
        "InProgress" => "In Progress",
        "Onhold" => "On Hold",
        "Completed" => "Completed",
        _ => "Other",
    }
}

fn column_subtitle(col: &str) -> &'static str {
    match col {
        "Pending" => "New & unassigned",
        "InProgress" => "Active support",
        "Onhold" => "Waiting on user/vendor",
        "Completed" => "Resolved & closed",
        _ => "",
    }
}

fn priority_badge(priority: &str) -> &'static str {
    match priority {
        "Critical" | "High" => "destructive",
        "Medium" => "secondary",
        _ => "outline",
    }
}

fn status_badge_class(status: &str) -> &'static str {
    match status {
        "Pending" => "text-warning border-warning/30",
        "InProgress" => "text-info border-info/30",
        "Onhold" => "text-muted-foreground border-border",
        "Completed" => "text-success border-success/30",
        _ => "text-primary border-primary/30",
    }
}

fn status_badge_variant(status: &str) -> &'static str {
    match status {
        "InProgress" => "secondary",
        _ => "outline",
    }
}

fn sla_badge_class(sla: &str) -> &'static str {
    match sla {
        "Within SLA" => "text-success border-success/30 bg-success/10",
        "Approaching Breach" => "text-warning border-warning/30 bg-warning/10",
        "Breached" => "text-destructive border-destructive/30 bg-destructive/10",
        _ => "text-muted-foreground border-border bg-muted/30",
    }
}

// ============================================================
// Context Menus
// ============================================================

#[derive(Props, Clone, PartialEq)]
struct TicketContextMenuProps {
    ticket: TicketItem,
    children: Element,
    on_view: EventHandler<TicketItem>,
    on_edit: EventHandler<TicketItem>,
}

#[component]
fn TicketContextMenu(props: TicketContextMenuProps) -> Element {
    let mut state = use_context::<TicketContextState>();
    let toast = use_context::<ToastState>();
    let t = props.ticket;
    let t_id = t.id.clone();
    let current_status = t.kanban_state.clone();
    let current_prio = t.priority.clone();

    rsx! {
        ContextMenu {
            ContextMenuTrigger {
                class: "contents",
                {props.children}
            }
            ContextMenuContent {
                class: "w-56 rounded-md shadow-xl border-border/80 bg-popover/95 backdrop-blur-md",
                ContextMenuLabel {
                    div { class: "flex items-center justify-between gap-2",
                        span { class: "font-mono font-bold text-foreground text-xs", "{t_id}" }
                        span { class: "rounded px-1.5 py-0.2 text-[10px] font-semibold bg-muted text-muted-foreground", "{t.priority}" }
                    }
                    span { class: "text-[11px] text-muted-foreground truncate block font-normal mt-0.5", "{t.emp_name()} • {t.emp_code_only()}" }
                }
                ContextMenuSeparator {}

                ContextMenuItem {
                    on_select: {
                        let t_c = t.clone();
                        let handler = props.on_view;
                        move |_| handler.call(t_c.clone())
                    },
                    EyeIcon { class: "size-3.5 text-muted-foreground" }
                    "View Ticket"
                }

                ContextMenuItem {
                    on_select: {
                        let t_c = t.clone();
                        let handler = props.on_edit;
                        move |_| handler.call(t_c.clone())
                    },
                    PencilIcon { class: "size-3.5 text-muted-foreground" }
                    "Modify Ticket"
                }

                ContextMenuSeparator {}

                ContextMenuSub {
                    ContextMenuSubTrigger {
                        "Change Status"
                    }
                    ContextMenuSubContent {
                        for &(code, label) in &[
                            ("Pending", "Pending"),
                            ("InProgress", "In Progress"),
                            ("Onhold", "On Hold"),
                            ("Completed", "Completed"),
                        ] {
                            {
                                let is_active = current_status == code;
                                let id_c = t_id.clone();
                                let label_str = label.to_string();
                                rsx! {
                                    ContextMenuItem {
                                        key: "status-{code}",
                                        on_select: move |_| {
                                            state.update_status(&id_c, code);
                                            toast.success("Status Updated", format!("{id_c} moved to {label_str}"));
                                        },
                                        div { class: "flex items-center justify-between w-full",
                                            span { "{label}" }
                                            if is_active {
                                                CheckIcon { class: "size-3.5 text-primary ml-auto" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                ContextMenuSub {
                    ContextMenuSubTrigger {
                        "Change Priority"
                    }
                    ContextMenuSubContent {
                        for &prio in &["Urgent", "High", "Medium", "Low"] {
                            {
                                let is_active = current_prio == prio;
                                let id_c = t_id.clone();
                                rsx! {
                                    ContextMenuItem {
                                        key: "prio-{prio}",
                                        on_select: move |_| {
                                            state.update_priority(&id_c, prio);
                                            toast.info("Priority Updated", format!("{id_c} priority set to {prio}"));
                                        },
                                        div { class: "flex items-center justify-between w-full",
                                            span { "{prio}" }
                                            if is_active {
                                                CheckIcon { class: "size-3.5 text-primary ml-auto" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                ContextMenuItem {
                    on_select: {
                        let id_c = t_id.clone();
                        move |_| {
                            toast.info("Copied", format!("{id_c} copied to clipboard"));
                        }
                    },
                    CopyIcon { class: "size-3.5 text-muted-foreground" }
                    "Copy Ticket ID"
                }

                ContextMenuSeparator {}

                ContextMenuItem {
                    destructive: true,
                    on_select: {
                        let id_c = t_id.clone();
                        move |_| {
                            state.delete_ticket(&id_c);
                            toast.error("Deleted", format!("{id_c} removed"));
                        }
                    },
                    TrashIcon { class: "size-3.5 text-destructive" }
                    "Delete Ticket"
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct TableFloatingContextMenuProps {
    ticket: TicketItem,
    open: Signal<bool>,
    x: Signal<f64>,
    y: Signal<f64>,
    on_view: EventHandler<TicketItem>,
    on_edit: EventHandler<TicketItem>,
}

#[component]
fn TableFloatingContextMenu(props: TableFloatingContextMenuProps) -> Element {
    let mut state = use_context::<TicketContextState>();
    let toast = use_context::<ToastState>();
    let t = props.ticket;
    let t_id = t.id.clone();
    let current_status = t.kanban_state.clone();
    let current_prio = t.priority.clone();

    use_context_provider(|| crate::components::ui::context_menu::ContextMenuState {
        open: props.open,
        x: props.x,
        y: props.y,
    });

    rsx! {
        ContextMenuContent {
            class: "w-56 rounded-md shadow-xl border-border/80 bg-popover/95 backdrop-blur-md",
            ContextMenuLabel {
                div { class: "flex items-center justify-between gap-2",
                    span { class: "font-mono font-bold text-foreground text-xs", "{t_id}" }
                    span { class: "rounded px-1.5 py-0.2 text-[10px] font-semibold bg-muted text-muted-foreground", "{t.priority}" }
                }
                span { class: "text-[11px] text-muted-foreground truncate block font-normal mt-0.5", "{t.emp_name()} • {t.emp_code_only()}" }
            }
            ContextMenuSeparator {}

            ContextMenuItem {
                on_select: {
                    let t_c = t.clone();
                    let handler = props.on_view;
                    move |_| handler.call(t_c.clone())
                },
                EyeIcon { class: "size-3.5 text-muted-foreground" }
                "View Ticket"
            }

            ContextMenuItem {
                on_select: {
                    let t_c = t.clone();
                    let handler = props.on_edit;
                    move |_| handler.call(t_c.clone())
                },
                PencilIcon { class: "size-3.5 text-muted-foreground" }
                "Modify Ticket"
            }

            ContextMenuSeparator {}

            ContextMenuSub {
                ContextMenuSubTrigger {
                    "Change Status"
                }
                ContextMenuSubContent {
                    for &(code, label) in &[
                        ("Pending", "Pending"),
                        ("InProgress", "In Progress"),
                        ("Onhold", "On Hold"),
                        ("Completed", "Completed"),
                    ] {
                        {
                            let is_active = current_status == code;
                            let id_c = t_id.clone();
                            let label_str = label.to_string();
                            rsx! {
                                ContextMenuItem {
                                    key: "status-{code}",
                                    on_select: move |_| {
                                        state.update_status(&id_c, code);
                                        toast.success("Status Updated", format!("{id_c} moved to {label_str}"));
                                    },
                                    div { class: "flex items-center justify-between w-full",
                                        span { "{label}" }
                                        if is_active {
                                            CheckIcon { class: "size-3.5 text-primary ml-auto" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            ContextMenuSub {
                ContextMenuSubTrigger {
                    "Change Priority"
                }
                ContextMenuSubContent {
                    for &prio in &["Urgent", "High", "Medium", "Low"] {
                        {
                            let is_active = current_prio == prio;
                            let id_c = t_id.clone();
                            rsx! {
                                ContextMenuItem {
                                    key: "prio-{prio}",
                                    on_select: move |_| {
                                        state.update_priority(&id_c, prio);
                                        toast.info("Priority Updated", format!("{id_c} priority set to {prio}"));
                                    },
                                    div { class: "flex items-center justify-between w-full",
                                        span { "{prio}" }
                                        if is_active {
                                            CheckIcon { class: "size-3.5 text-primary ml-auto" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            ContextMenuItem {
                on_select: {
                    let id_c = t_id.clone();
                    move |_| {
                        toast.info("Copied", format!("{id_c} copied to clipboard"));
                    }
                },
                CopyIcon { class: "size-3.5 text-muted-foreground" }
                "Copy Ticket ID"
            }

            ContextMenuSeparator {}

            ContextMenuItem {
                destructive: true,
                on_select: {
                    let id_c = t_id.clone();
                    move |_| {
                        state.delete_ticket(&id_c);
                        toast.error("Deleted", format!("{id_c} removed"));
                    }
                },
                TrashIcon { class: "size-3.5 text-destructive" }
                "Delete Ticket"
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct ColumnContextMenuProps {
    column: String,
    children: Element,
    on_quick_add: EventHandler<String>,
    on_filter_column: EventHandler<String>,
    on_clear_filters: EventHandler<()>,
}

#[component]
fn ColumnContextMenu(props: ColumnContextMenuProps) -> Element {
    let col = props.column;
    let col_disp = column_display_label(&col);

    rsx! {
        ContextMenu {
            ContextMenuTrigger {
                class: "contents",
                {props.children}
            }
            ContextMenuContent {
                class: "w-52 rounded-md shadow-xl border-border/80 bg-popover/95 backdrop-blur-md",
                ContextMenuLabel {
                    span { class: "font-semibold text-foreground text-xs", "Column: {col_disp}" }
                }
                ContextMenuSeparator {}
                ContextMenuItem {
                    on_select: {
                        let col_c = col.clone();
                        let handler = props.on_quick_add;
                        move |_| handler.call(col_c.clone())
                    },
                    PlusIcon { class: "size-3.5 text-primary" }
                    "Quick Add Ticket"
                }
                ContextMenuItem {
                    on_select: {
                        let col_c = col.clone();
                        let handler = props.on_filter_column;
                        move |_| handler.call(col_c.clone())
                    },
                    FilterIcon { class: "size-3.5 text-muted-foreground" }
                    "Filter: Only {col_disp}"
                }
                ContextMenuItem {
                    on_select: {
                        let handler = props.on_clear_filters;
                        move |_| handler.call(())
                    },
                    "Clear Filters"
                }
            }
        }
    }
}

// ============================================================
// DataTable Cell Renderers
// ============================================================

#[component]
fn TicketIdCell(ticket: TicketItem) -> Element {
    let mut state = use_context::<TicketContextState>();
    let id = ticket.id.clone();
    let t_view = ticket.clone();
    let t_edit = ticket.clone();
    rsx! {
        TicketContextMenu {
            ticket: ticket.clone(),
            on_view: move |_| state.open_view(t_view.clone()),
            on_edit: move |_| state.open_edit(t_edit.clone()),
            span {
                class: "font-mono font-bold text-foreground text-xs hover:text-primary transition-colors cursor-context-menu",
                "{id}"
            }
        }
    }
}

#[component]
fn TicketTitleCell(ticket: TicketItem) -> Element {
    let mut state = use_context::<TicketContextState>();
    let title = ticket.title.clone();
    let emp_name = ticket.emp_name();
    let emp_code = ticket.emp_code_only().to_string();
    let email = ticket.email_address.clone();
    let t_view = ticket.clone();
    let t_edit = ticket.clone();
    rsx! {
        TicketContextMenu {
            ticket: ticket.clone(),
            on_view: move |_| state.open_view(t_view.clone()),
            on_edit: move |_| state.open_edit(t_edit.clone()),
            div { class: "flex flex-col min-w-0 py-0.5 cursor-context-menu",
                span { class: "font-semibold text-foreground truncate text-xs hover:text-primary transition-colors", "{title}" }
                div { class: "flex items-center gap-1.5 text-[11px] text-muted-foreground truncate",
                    span { class: "font-medium text-foreground/80", "{emp_name}" }
                    span { "•" }
                    span { class: "font-mono text-[10px]", "{emp_code}" }
                    span { "•" }
                    span { class: "truncate opacity-80", "{email}" }
                }
            }
        }
    }
}

#[component]
fn TicketActionsCell(ticket: TicketItem) -> Element {
    let mut state = use_context::<TicketContextState>();
    let toast = use_context::<ToastState>();
    let t_view = ticket.clone();
    let t_edit = ticket.clone();
    let t_del = ticket.clone();
    let t_copy = ticket.clone();
    rsx! {
        div { class: "flex items-center gap-1",
            button {
                r#type: "button",
                title: "View Ticket Details",
                class: "cursor-pointer inline-flex size-7 items-center justify-center rounded-sm text-muted-foreground hover:text-foreground hover:bg-muted/80 transition-colors",
                onclick: move |e: MouseEvent| {
                    e.stop_propagation();
                    state.open_view(t_view.clone());
                },
                EyeIcon { class: "size-3.5" }
            }
            button {
                r#type: "button",
                title: "Modify Ticket",
                class: "cursor-pointer inline-flex size-7 items-center justify-center rounded-sm text-muted-foreground hover:text-foreground hover:bg-muted/80 transition-colors",
                onclick: move |e: MouseEvent| {
                    e.stop_propagation();
                    state.open_edit(t_edit.clone());
                },
                PencilIcon { class: "size-3.5" }
            }
            DropdownMenu {
                DropdownMenuTrigger {
                    class: "cursor-pointer inline-flex size-7 items-center justify-center rounded-sm text-muted-foreground hover:text-foreground hover:bg-muted/80 transition-colors",
                    MoreHorizontalIcon { class: "size-3.5" }
                }
                DropdownMenuContent {
                    class: "w-44 rounded-md shadow-xl border-border/80 bg-popover/95 backdrop-blur-md",
                    align: "end".to_string(),
                    DropdownMenuLabel { "Actions" }
                    DropdownMenuSeparator {}
                    DropdownMenuItem {
                        onclick: {
                            let mut state = state;
                            let t = t_view.clone();
                            move |_| state.open_view(t.clone())
                        },
                        EyeIcon { class: "size-3.5 text-muted-foreground mr-1.5" }
                        "View Details"
                    }
                    DropdownMenuItem {
                        onclick: {
                            let mut state = state;
                            let t = t_edit.clone();
                            move |_| state.open_edit(t.clone())
                        },
                        PencilIcon { class: "size-3.5 text-muted-foreground mr-1.5" }
                        "Edit Ticket"
                    }
                    DropdownMenuItem {
                        onclick: {
                            let id = t_copy.id.clone();
                            move |_| {
                                toast.info("Copied", format!("{id} copied to clipboard"));
                            }
                        },
                        CopyIcon { class: "size-3.5 text-muted-foreground mr-1.5" }
                        "Copy ID"
                    }
                    DropdownMenuSeparator {}
                    DropdownMenuItem {
                        variant: "destructive".to_string(),
                        onclick: {
                            let mut state = state;
                            let id = t_del.id.clone();
                            move |_| {
                                state.delete_ticket(&id);
                                toast.error("Deleted", format!("{id} removed"));
                            }
                        },
                        TrashIcon { class: "size-3.5 text-destructive mr-1.5" }
                        "Delete"
                    }
                }
            }
        }
    }
}

fn render_id_cell(t: &TicketItem) -> Element {
    rsx! {
        TicketIdCell { ticket: t.clone() }
    }
}

fn render_title_cell(t: &TicketItem) -> Element {
    rsx! {
        TicketTitleCell { ticket: t.clone() }
    }
}

fn render_actions_cell(t: &TicketItem) -> Element {
    rsx! {
        TicketActionsCell { ticket: t.clone() }
    }
}

fn render_category_cell(t: &TicketItem) -> Element {
    let cat = t.category.clone();
    rsx! {
        span { class: "rounded-md bg-muted px-2 py-0.5 text-[10px] font-medium text-muted-foreground",
            "{cat}"
        }
    }
}

fn render_status_cell(t: &TicketItem) -> Element {
    let status = t.status.clone();
    let variant = status_badge_variant(&status);
    let class = status_badge_class(&status);
    let label = column_display_label(&status);
    rsx! {
        Badge {
            variant: "{variant}",
            class: "text-[10px] font-semibold {class}",
            "{label}"
        }
    }
}

fn render_sla_cell(t: &TicketItem) -> Element {
    let sla = t.sla_status.clone();
    let class = sla_badge_class(&sla);
    rsx! {
        if !sla.is_empty() {
            span { class: "rounded px-1.5 py-0.5 text-[10px] font-semibold border {class}",
                "{sla}"
            }
        } else {
            span { class: "text-muted-foreground/60 text-xs", "—" }
        }
    }
}

fn render_priority_cell(t: &TicketItem) -> Element {
    let priority = t.priority.clone();
    let variant = priority_badge(&priority);
    rsx! {
        Badge {
            variant: "{variant}",
            class: "text-[10px] px-1.5 py-0.2 font-semibold",
            "{priority}"
        }
    }
}

fn render_assignee_cell(t: &TicketItem) -> Element {
    let assignee = t.assignee.clone();
    rsx! {
        div { class: "flex items-center gap-1.5",
            div { class: "flex size-6 items-center justify-center rounded-full bg-muted text-[10px] font-bold text-foreground border border-border/80",
                "{assignee}"
            }
        }
    }
}

fn render_created_cell(t: &TicketItem) -> Element {
    let created = t.created_at.clone();
    rsx! {
        span { class: "text-[11px] text-muted-foreground font-mono", "{created}" }
    }
}

fn ticket_row_id(t: &TicketItem) -> String {
    t.id.clone()
}

fn ticket_search_text(t: &TicketItem) -> String {
    format!(
        "{} {} {} {} {} {} {} {} {}",
        t.id, t.subject, t.employee_code, t.email_address, t.category, t.priority, t.assigned_to, t.customer, t.sla_status
    )
}

fn ticket_filter_value(t: &TicketItem) -> String {
    t.status.clone()
}

fn ticket_priority_filter_value(t: &TicketItem) -> String {
    t.priority.clone()
}

fn ticket_columns() -> Vec<DataTableColumn<TicketItem>> {
    vec![
        DataTableColumn::new(
            "id",
            "Ticket ID",
            render_id_cell,
            |t: &TicketItem| SortValue::Text(t.id.clone()),
        ),
        DataTableColumn::new(
            "title",
            "Subject & Requester",
            render_title_cell,
            |t: &TicketItem| SortValue::Text(t.title.clone()),
        ),
        DataTableColumn::new(
            "category",
            "Category",
            render_category_cell,
            |t: &TicketItem| SortValue::Text(t.category.clone()),
        ),
        DataTableColumn::new(
            "status",
            "Status",
            render_status_cell,
            |t: &TicketItem| SortValue::Text(t.status.clone()),
        ),
        DataTableColumn::new(
            "sla",
            "SLA Status",
            render_sla_cell,
            |t: &TicketItem| SortValue::Text(t.sla_status.clone()),
        ),
        DataTableColumn::new(
            "priority",
            "Priority",
            render_priority_cell,
            |t: &TicketItem| SortValue::Text(t.priority.clone()),
        ),
        DataTableColumn::new(
            "assignee",
            "Assignee",
            render_assignee_cell,
            |t: &TicketItem| SortValue::Text(t.assignee.clone()),
        ),
        DataTableColumn::new(
            "created",
            "Reported",
            render_created_cell,
            |t: &TicketItem| SortValue::Text(t.created_at.clone()),
        ),
        DataTableColumn::actions("actions", "Actions", render_actions_cell),
    ]
}

#[component]
pub fn Tickets() -> Element {
    let mut tickets = use_signal(initial_tickets);
    let mut search_query = use_signal(String::new);
    let mut selected_priority = use_signal(|| "All".to_string());

    // Drag-and-drop reactive state
    let mut dragged_id = use_signal(|| Option::<String>::None);
    let mut drag_over_col = use_signal(|| Option::<String>::None);
    let mut drag_over_card = use_signal(|| Option::<String>::None);

    // View & Edit Ticket Modal signals
    let viewing_ticket = use_signal(|| Option::<TicketItem>::None);
    let mut editing_ticket = use_signal(|| Option::<TicketItem>::None);
    let mut is_view_open = use_signal(|| false);
    let mut is_edit_open = use_signal(|| false);

    let mut edit_emp_name = use_signal(String::new);
    let mut edit_employee_code = use_signal(String::new);
    let mut edit_email_address = use_signal(String::new);
    let mut edit_category = use_signal(|| "General".to_string());
    let mut edit_subject = use_signal(String::new);
    let mut edit_problem_description = use_signal(String::new);
    let edit_attachment_name = use_signal(String::new);
    let edit_attachment_url = use_signal(String::new);
    let mut edit_priority = use_signal(|| "Medium".to_string());
    let mut edit_assigned_to = use_signal(|| "GR".to_string());
    let edit_customer = use_signal(|| "Internal".to_string());
    let edit_time_spent = use_signal(|| 0.0f64);
    let edit_activities = use_signal(String::new);
    let edit_my_activity = use_signal(String::new);
    let mut edit_deadline = use_signal(String::new);
    let mut edit_sla_status = use_signal(|| "Within SLA".to_string());
    let edit_sla_deadline = use_signal(String::new);
    let edit_rating_avg_text = use_signal(String::new);
    let mut edit_kanban_state = use_signal(|| "Pending".to_string());
    let mut edit_stage = use_signal(|| "Pending".to_string());
    let mut edit_status = use_signal(|| "Pending".to_string());

    // Table row floating context menu signals
    let mut table_ctx_open = use_signal(|| false);
    let mut table_ctx_x = use_signal(|| 0.0f64);
    let mut table_ctx_y = use_signal(|| 0.0f64);
    let mut table_ctx_ticket = use_signal(|| Option::<TicketItem>::None);

    let mut state = TicketContextState {
        tickets,
        viewing_ticket,
        editing_ticket,
        is_view_open,
        is_edit_open,
        edit_emp_name,
        edit_employee_code,
        edit_email_address,
        edit_category,
        edit_subject,
        edit_problem_description,
        edit_attachment_name,
        edit_attachment_url,
        edit_priority,
        edit_assigned_to,
        edit_customer,
        edit_time_spent,
        edit_activities,
        edit_my_activity,
        edit_deadline,
        edit_sla_status,
        edit_sla_deadline,
        edit_rating_avg_text,
        edit_kanban_state,
        edit_stage,
        edit_status,
    };
    use_context_provider(|| state);

    let toast = use_context::<ToastState>();

    let q = search_query.read().to_lowercase();
    let filter_prio = selected_priority.read().clone();

    // Move ticket to end of a column
    let mut move_to_column_end = move |src_id: String, target_col: String| {
        let mut list = tickets.write();
        if let Some(pos) = list.iter().position(|t| t.id == src_id) {
            let mut item = list.remove(pos);
            let old_status = item.status.clone();
            item.status = target_col.clone();
            item.kanban_state = target_col.clone();
            item.stage = target_col.clone();
            list.push(item);

            // Persist status change to SQLite management.db
            if let Some(num) = src_id.strip_prefix("IT-").and_then(|s| s.parse::<i64>().ok()) {
                let _ = crate::db::update_ticket_status(num, &target_col);
            }

            if old_status != target_col {
                let label = column_display_label(&target_col);
                toast.success("Status Updated", format!("{src_id} moved to {label}"));
            }
        }
    };

    // Move ticket before a specific target card in a column
    let mut move_card_before = move |src_id: String, target_card_id: String, target_col: String| {
        if src_id == target_card_id {
            return;
        }
        let mut list = tickets.write();
        if let Some(src_pos) = list.iter().position(|t| t.id == src_id) {
            let mut item = list.remove(src_pos);
            let old_status = item.status.clone();
            item.status = target_col.clone();
            item.kanban_state = target_col.clone();
            item.stage = target_col.clone();
            if let Some(target_pos) = list.iter().position(|t| t.id == target_card_id) {
                list.insert(target_pos, item);
            } else {
                list.push(item);
            }

            // Persist status change to SQLite management.db
            if let Some(num) = src_id.strip_prefix("IT-").and_then(|s| s.parse::<i64>().ok()) {
                let _ = crate::db::update_ticket_status(num, &target_col);
            }

            if old_status != target_col {
                let label = column_display_label(&target_col);
                toast.success("Moved & Reordered", format!("{src_id} moved to {label}"));
            } else {
                toast.info("Ticket Reordered", format!("{src_id} repositioned"));
            }
        }
    };

    // Reset drag signals helper
    let mut clear_drag_state = move || {
        dragged_id.set(None);
        drag_over_col.set(None);
        drag_over_card.set(None);
    };

    // Save changes from Edit / Modify Ticket dialog
    let mut save_edit = move || {
        if let Some(mut current) = editing_ticket.read().clone() {
            let subj_val = edit_subject.read().trim().to_string();
            if subj_val.is_empty() {
                toast.error("Validation Error", "Ticket subject cannot be empty");
                return;
            }

            let code_val = edit_employee_code.read().trim().to_string();
            let name_val = edit_emp_name.read().trim().to_string();
            let emp_combined = if name_val.is_empty() {
                code_val.clone()
            } else if code_val.is_empty() {
                name_val.clone()
            } else {
                format!("{} ({})", code_val, name_val)
            };

            let email_val = edit_email_address.read().trim().to_string();
            let cat_val = edit_category.read().clone();
            let desc_val = edit_problem_description.read().clone();
            let att_name_val = edit_attachment_name.read().clone();
            let att_url_val = edit_attachment_url.read().clone();
            let prio_val = edit_priority.read().clone();
            let assigned_val = edit_assigned_to.read().clone();
            let cust_val = edit_customer.read().clone();
            let time_val = *edit_time_spent.read();
            let act_val = edit_activities.read().clone();
            let my_act_val = edit_my_activity.read().clone();
            let deadline_val = edit_deadline.read().clone();
            let sla_stat_val = edit_sla_status.read().clone();
            let sla_dead_val = edit_sla_deadline.read().clone();
            let rating_val = edit_rating_avg_text.read().clone();

            // The stage represents the Kanban board stages
            let kanban_val = edit_kanban_state.read().clone();
            let stage_val = kanban_val.clone();
            let status_val = kanban_val.clone();

            current.subject = subj_val.clone();
            current.title = subj_val.clone();
            current.employee_code = emp_combined.clone();
            current.employee = emp_combined.clone();
            current.email_address = email_val.clone();
            current.category = cat_val.clone();
            current.problem_description = desc_val.clone();
            current.attachment_name = att_name_val.clone();
            current.attachment_url = att_url_val.clone();
            current.priority = prio_val.clone();
            current.assigned_to = assigned_val.clone();
            current.assignee = assigned_val.clone();
            current.customer = cust_val.clone();
            current.time_spent = time_val;
            current.activities = act_val.clone();
            current.my_activity = my_act_val.clone();
            current.deadline = deadline_val.clone();
            current.sla_status = sla_stat_val.clone();
            current.sla_deadline = sla_dead_val.clone();
            current.rating_avg_text = rating_val.clone();
            current.kanban_state = kanban_val.clone();
            current.status = kanban_val.clone();
            current.stage = stage_val.clone();
            current.last_updated_on = "Just now".to_string();

            let mut list = tickets.write();
            if let Some(pos) = list.iter().position(|t| t.id == current.id) {
                list[pos] = current.clone();
            }

            if let Some(num) = current.id.strip_prefix("IT-").and_then(|s| s.parse::<i64>().ok()) {
                let _ = crate::db::update_ticket_full(
                    num,
                    &emp_combined,
                    &email_val,
                    &cat_val,
                    &subj_val,
                    &desc_val,
                    &att_name_val,
                    &att_url_val,
                    &prio_val,
                    &assigned_val,
                    &cust_val,
                    time_val,
                    &act_val,
                    &my_act_val,
                    &deadline_val,
                    &sla_stat_val,
                    &sla_dead_val,
                    &rating_val,
                    &kanban_val,
                    &stage_val,
                    &status_val,
                );
            }

            toast.success("Ticket Modified", format!("{} modified successfully", current.id));
            state.is_edit_open.set(false);
            state.editing_ticket.set(None);
        }
    };

    // Quick add ticket to column
    let mut quick_add_ticket = move |col: String| {
        let count = tickets.read().len() + 1;
        let fallback_id = format!("IT-{}", 200 + count);
        let col_disp = column_display_label(&col);
        let title = format!("New request ({col_disp})");
        let employee = "EMP-1001 (CurrentUser)".to_string();
        let category = "General".to_string();
        let priority = "Medium".to_string();
        let assignee = "GR".to_string();
        let created_at = "Just now".to_string();

        let db_id = crate::db::create_ticket(&title, &category, &priority, &employee, &assignee, &col, "Quick ticket created via column quick-add action");
        let real_id = if let Ok(id) = db_id { format!("IT-{id}") } else { fallback_id };
        let numeric_id = db_id.unwrap_or(200 + count as i64);

        tickets.write().push(TicketItem::new(
            numeric_id,
            &employee,
            "admin@taskflow.internal",
            &category,
            &title,
            "Quick ticket created via column quick-add action",
            "",
            "",
            &priority,
            &assignee,
            "Internal",
            0.0,
            "Ticket created",
            "Initial assignment to team lead",
            "",
            "Within SLA",
            "",
            "",
            &col,
            "Triage",
            "Open",
            &created_at,
            &created_at,
        ));
        toast.success("Ticket Created", format!("{real_id} created in {col_disp}"));
    };


    rsx! {
        div { class: "flex flex-col flex-1 w-full h-full p-4 md:p-6 min-h-0 overflow-hidden",

            // Page header
            div { class: "flex items-center justify-between border-b border-border/50 pb-3 mb-3 shrink-0",
                div { class: "flex items-center gap-3",
                    h1 { class: "text-xl font-bold tracking-tight text-foreground sm:text-2xl", "IT Tickets" }
                    Badge {
                        variant: "secondary",
                        class: "font-semibold",
                        "{tickets.read().len()} tickets"
                    }
                }
            }

            Tabs {
                value: "board".to_string(),
                class: "flex flex-col flex-1 min-h-0 w-full overflow-hidden",
                TabsList {
                    TabsTrigger { value: "board".to_string(), "Kanban Board" }
                    TabsTrigger { value: "table".to_string(), "IT Data Table" }
                }

                // --------------------------------------------------------
                // Kanban Board View (Onhold, pending, InProgress, Completed)
                // --------------------------------------------------------
                TabsContent {
                    value: "board".to_string(),
                    class: "flex flex-col flex-1 min-h-0 w-full overflow-hidden gap-3 pt-2",

                    // Search & Filter toolbar for Kanban
                    div { class: "shrink-0 flex flex-wrap items-center justify-between gap-3",
                        div { class: "relative flex flex-1 min-w-[200px] max-w-sm items-center",
                            div { class: "pointer-events-none absolute left-3 flex items-center text-muted-foreground",
                                SearchIcon { class: "size-4" }
                            }
                            Input {
                                class: "pl-9 text-xs h-9 w-full",
                                placeholder: "Filter tickets by ID, title, requester, or category...".to_string(),
                                value: search_query.read().clone(),
                                oninput: move |e: FormEvent| search_query.set(e.value()),
                            }
                        }

                        div { class: "flex items-center gap-1.5",
                            span { class: "text-xs font-medium text-muted-foreground mr-1", "Priority:" }
                            for prio in ["All", "High", "Medium", "Low"] {
                                {
                                    let is_sel = filter_prio == prio;
                                    let btn_variant = if is_sel { "default" } else { "outline" };
                                    rsx! {
                                        Button {
                                            variant: "{btn_variant}",
                                            size: "sm",
                                            key: "prio-{prio}",
                                            class: "h-7 px-2.5 text-xs",
                                            onclick: move |_| selected_priority.set(prio.to_string()),
                                            "{prio}"
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // 4 Kanban Columns side-by-side in full width
                    div { class: "grid grid-cols-1 md:grid-cols-2 xl:grid-cols-4 gap-3.5 w-full flex-1 min-h-0 overflow-hidden",
                            for &column in COLUMN_NAMES {
                                {
                                    let col_display = column_display_label(column);
                                    let col_sub = column_subtitle(column);
                                    let col_dot = match column {
                                        "InProgress" => "bg-primary animate-pulse",
                                        "Pending" => "bg-warning",
                                        "Onhold" => "bg-muted-foreground",
                                        _ => "bg-success",
                                    };

                                    let all_tickets = tickets.read().clone();
                                    let column_tickets: Vec<TicketItem> = all_tickets
                                        .into_iter()
                                        .filter(|t| {
                                            let st = if !t.kanban_state.is_empty() {
                                                t.kanban_state.as_str()
                                            } else if !t.stage.is_empty() {
                                                t.stage.as_str()
                                            } else {
                                                t.status.as_str()
                                            };
                                            match (st, column) {
                                                ("InProgress", "InProgress") | ("In Progress", "InProgress") => true,
                                                ("Pending", "Pending") | ("Open", "Pending") => true,
                                                ("Onhold", "Onhold") | ("On Hold", "Onhold") => true,
                                                ("Completed", "Completed") | ("Resolved", "Completed") | ("Closed", "Completed") => true,
                                                (s, c) => s == c || s.eq_ignore_ascii_case(c),
                                            }
                                        })
                                        .filter(|t| {
                                            let match_q = q.is_empty()
                                                || t.id.to_lowercase().contains(&q)
                                                || t.title.to_lowercase().contains(&q)
                                                || t.employee.to_lowercase().contains(&q)
                                                || t.emp_name().to_lowercase().contains(&q)
                                                || t.emp_code_only().to_lowercase().contains(&q)
                                                || t.email_address.to_lowercase().contains(&q)
                                                || t.category.to_lowercase().contains(&q)
                                                || t.assignee.to_lowercase().contains(&q);
                                            let match_prio = filter_prio == "All" || t.priority == filter_prio;
                                            match_q && match_prio
                                        })
                                        .collect();

                                    let is_col_over = drag_over_col.read().as_deref() == Some(column);
                                    let col_highlight = if is_col_over {
                                        "ring-1 ring-primary border-primary bg-primary/5"
                                    } else {
                                        ""
                                    };

                                    rsx! {
                                        ColumnContextMenu {
                                            column: column.to_string(),
                                            on_quick_add: move |col| quick_add_ticket(col),
                                            on_filter_column: move |col| search_query.set(col),
                                            on_clear_filters: move |_| {
                                                search_query.set(String::new());
                                                selected_priority.set("All".to_string());
                                            },

                                            div {
                                                key: "board-col-{column}",
                                                class: "flex flex-col h-full min-h-0 rounded-md border border-border/70 bg-card/75 p-3 transition-colors duration-150 {col_highlight}",
                                                ondragenter: move |e: DragEvent| {
                                                    e.prevent_default();
                                                    if drag_over_col.read().as_deref() != Some(column) {
                                                        drag_over_col.set(Some(column.to_string()));
                                                    }
                                                },
                                                ondragover: move |e: DragEvent| {
                                                    e.prevent_default();
                                                    if drag_over_col.read().as_deref() != Some(column) {
                                                        drag_over_col.set(Some(column.to_string()));
                                                    }
                                                },
                                                ondrop: move |e: DragEvent| {
                                                    e.prevent_default();
                                                    let src = e.data().data_transfer().get_as_text()
                                                        .filter(|s| !s.is_empty())
                                                        .or_else(|| dragged_id.read().clone());
                                                    if let Some(s) = src {
                                                        move_to_column_end(s, column.to_string());
                                                    }
                                                    clear_drag_state();
                                                },

                                                // Column header
                                                div { class: "flex items-center justify-between px-1 border-b border-border/40 pb-2.5",
                                                    div { class: "flex flex-col gap-0.5",
                                                        div { class: "flex items-center gap-2",
                                                            span { class: "size-2 rounded-full {col_dot}" }
                                                            h2 { class: "text-xs font-bold uppercase tracking-wider text-foreground",
                                                                "{col_display}"
                                                            }
                                                        }
                                                        span { class: "text-[11px] text-muted-foreground", "{col_sub}" }
                                                    }
                                                    Badge { variant: "outline", class: "text-[10px] font-mono tabular-nums text-muted-foreground",
                                                        "{column_tickets.len()}"
                                                    }
                                                }

                                                // Column cards container (acts as full drop target)
                                                div {
                                                    class: "flex flex-col gap-2 min-h-0 flex-1 overflow-y-auto pr-1",
                                                    ondragenter: move |e: DragEvent| {
                                                        e.prevent_default();
                                                        if drag_over_col.read().as_deref() != Some(column) {
                                                            drag_over_col.set(Some(column.to_string()));
                                                        }
                                                    },
                                                    ondragover: move |e: DragEvent| {
                                                        e.prevent_default();
                                                        if drag_over_col.read().as_deref() != Some(column) {
                                                            drag_over_col.set(Some(column.to_string()));
                                                        }
                                                    },
                                                    ondrop: move |e: DragEvent| {
                                                        e.prevent_default();
                                                        let src = e.data().data_transfer().get_as_text()
                                                            .filter(|s| !s.is_empty())
                                                            .or_else(|| dragged_id.read().clone());
                                                        if let Some(s) = src {
                                                            move_to_column_end(s, column.to_string());
                                                        }
                                                        clear_drag_state();
                                                    },

                                                    for ticket in column_tickets.iter() {
                                                        {
                                                            let id = ticket.id.clone();
                                                            let title = ticket.title.clone();
                                                            let priority = ticket.priority.clone();
                                                            let category = ticket.category.clone();
                                                            let assignee = ticket.assignee.clone();
                                                            let ticket_status = ticket.status.clone();
                                                            let sla_status = ticket.sla_status.clone();
                                                            let deadline = ticket.deadline.clone();

                                                            let is_dragged = dragged_id.read().as_deref() == Some(&id);
                                                            let is_card_over = drag_over_card.read().as_deref() == Some(&id) && !is_dragged;

                                                            let card_cls = if is_dragged {
                                                                "opacity-40 scale-[0.99] border-dashed border-primary ring-1 ring-primary/40 bg-muted/40 cursor-grabbing rounded-md border p-3 flex flex-col gap-2"
                                                            } else if ticket_status == "Completed" {
                                                                "group flex cursor-grab active:cursor-grabbing flex-col gap-2 rounded-md border border-success/30 bg-card p-3 transition-colors duration-150 hover:border-success/60 hover:bg-success/5 shadow-xs"
                                                            } else if ticket_status == "InProgress" {
                                                                "group flex cursor-grab active:cursor-grabbing flex-col gap-2 rounded-md border border-primary/40 bg-card p-3 transition-colors duration-150 hover:border-primary/70 hover:bg-primary/5 shadow-xs"
                                                            } else {
                                                                "group flex cursor-grab active:cursor-grabbing flex-col gap-2 rounded-md border border-border/70 bg-card p-3 transition-colors duration-150 hover:border-primary/50 hover:bg-accent/20 shadow-xs"
                                                            };

                                                            let id_for_start = id.clone();
                                                            let id_for_enter = id.clone();
                                                            let id_for_over = id.clone();
                                                            let id_for_drop = id.clone();

                                                            rsx! {
                                                                div {
                                                                    key: "board-card-wrap-{id}",
                                                                    class: "flex flex-col gap-1.5",
                                                                    ondragenter: move |e: DragEvent| {
                                                                        e.prevent_default();
                                                                        if drag_over_card.read().as_deref() != Some(&id_for_enter) {
                                                                            drag_over_card.set(Some(id_for_enter.clone()));
                                                                        }
                                                                        if drag_over_col.read().as_deref() != Some(column) {
                                                                            drag_over_col.set(Some(column.to_string()));
                                                                        }
                                                                    },
                                                                    ondragover: move |e: DragEvent| {
                                                                        e.prevent_default();
                                                                        if drag_over_card.read().as_deref() != Some(&id_for_over) {
                                                                            drag_over_card.set(Some(id_for_over.clone()));
                                                                        }
                                                                        if drag_over_col.read().as_deref() != Some(column) {
                                                                            drag_over_col.set(Some(column.to_string()));
                                                                        }
                                                                    },
                                                                    ondrop: move |e: DragEvent| {
                                                                        e.prevent_default();
                                                                        e.stop_propagation();
                                                                        let src = e.data().data_transfer().get_as_text()
                                                                            .filter(|s| !s.is_empty())
                                                                            .or_else(|| dragged_id.read().clone());
                                                                        if let Some(s) = src {
                                                                            move_card_before(s, id_for_drop.clone(), column.to_string());
                                                                        }
                                                                        clear_drag_state();
                                                                    },

                                                                    // Insertion indicator before card
                                                                    if is_card_over {
                                                                        div { class: "h-1 w-full rounded-full bg-primary animate-pulse my-0.5" }
                                                                    }

                                                                    TicketContextMenu {
                                                                        ticket: ticket.clone(),
                                                                        on_view: move |t| state.open_view(t),
                                                                        on_edit: move |t| state.open_edit(t),

                                                                        div {
                                                                            draggable: "true",
                                                                            "data-drag-id": "{id}",
                                                                            class: "select-none {card_cls}",
                                                                            ondragstart: move |_| {
                                                                                dragged_id.set(Some(id_for_start.clone()));
                                                                            },
                                                                            ondragend: move |_| {
                                                                                clear_drag_state();
                                                                            },
                                                                            ondoubleclick: {
                                                                                let t_dbl = ticket.clone();
                                                                                move |_| state.open_view(t_dbl.clone())
                                                                            },

                                                                            // Inner card content
                                                                            div { class: "pointer-events-none flex flex-col gap-2 w-full",
                                                                                // Header: ID, Status badge, Priority badge, and direct View & Modify buttons!
                                                                                div { class: "flex items-center justify-between gap-1.5",
                                                                                    div { class: "flex items-center gap-1.5 min-w-0",
                                                                                        GripVerticalIcon { class: "size-3.5 shrink-0 text-muted-foreground/40 group-hover:text-primary transition-colors cursor-grab" }
                                                                                        span { class: "font-mono text-[11px] font-bold text-muted-foreground group-hover:text-primary transition-colors",
                                                                                            "{id}"
                                                                                        }
                                                                                    }
                                                                                    div { class: "flex items-center gap-1 shrink-0",
                                                                                        Badge {
                                                                                            variant: "{status_badge_variant(&ticket_status)}",
                                                                                            class: "text-[10px] font-semibold {status_badge_class(&ticket_status)}",
                                                                                            "{column_display_label(&ticket_status)}"
                                                                                        }
                                                                                        Badge {
                                                                                            variant: "{priority_badge(&priority)}",
                                                                                            class: "text-[10px] px-1.5 py-0.2",
                                                                                            "{priority}"
                                                                                        }
                                                                                        // Direct View & Modify buttons on the Kanban card
                                                                                        div { class: "flex items-center gap-0.5 pointer-events-auto ml-1 border-l border-border/50 pl-1",
                                                                                            button {
                                                                                                r#type: "button",
                                                                                                title: "View Ticket Details",
                                                                                                class: "cursor-pointer rounded p-1 text-muted-foreground hover:text-primary hover:bg-muted/80 transition-colors",
                                                                                                onclick: {
                                                                                                    let t_v = ticket.clone();
                                                                                                    move |e: MouseEvent| {
                                                                                                        e.stop_propagation();
                                                                                                        state.open_view(t_v.clone());
                                                                                                    }
                                                                                                },
                                                                                                EyeIcon { class: "size-3.5" }
                                                                                            }
                                                                                            button {
                                                                                                r#type: "button",
                                                                                                title: "Modify Ticket",
                                                                                                class: "cursor-pointer rounded p-1 text-muted-foreground hover:text-primary hover:bg-muted/80 transition-colors",
                                                                                                onclick: {
                                                                                                    let t_e = ticket.clone();
                                                                                                    move |e: MouseEvent| {
                                                                                                        e.stop_propagation();
                                                                                                        state.open_edit(t_e.clone());
                                                                                                    }
                                                                                                },
                                                                                                PencilIcon { class: "size-3.5" }
                                                                                            }
                                                                                        }
                                                                                    }
                                                                                }

                                                                                p { class: "text-sm font-semibold leading-snug text-foreground group-hover:text-primary/95 transition-colors line-clamp-2",
                                                                                    "{title}"
                                                                                }

                                                                                // Requester identity: Name, Code & Email
                                                                                div { class: "flex flex-col gap-0.5 text-[11px] pt-0.5",
                                                                                    div { class: "flex items-center justify-between gap-1.5 text-muted-foreground",
                                                                                        span { class: "font-medium text-foreground truncate", "{ticket.emp_name()}" }
                                                                                        span { class: "font-mono text-[10px] text-muted-foreground shrink-0", "{ticket.emp_code_only()}" }
                                                                                    }
                                                                                    div { class: "flex items-center justify-between gap-1 text-[10px] text-muted-foreground/80",
                                                                                        span { class: "truncate", "{ticket.email_address}" }
                                                                                        if !sla_status.is_empty() {
                                                                                            span { class: "rounded px-1.5 py-0.2 text-[9px] font-semibold {sla_badge_class(&sla_status)} shrink-0",
                                                                                                "{sla_status}"
                                                                                            }
                                                                                        }
                                                                                    }
                                                                                }

                                                                                if !deadline.is_empty() {
                                                                                    div { class: "flex items-center justify-between text-[10px] text-muted-foreground/80 pt-1 border-t border-border/30",
                                                                                        span { class: "font-mono text-warning font-medium", "Due: {deadline}" }
                                                                                    }
                                                                                }
                                                                            }

                                                                            // Footer with quick state transition buttons & assignee
                                                                            div { class: "mt-1 flex items-center justify-between pt-2 border-t border-border/40 text-xs",
                                                                                div { class: "flex items-center gap-1 pointer-events-auto",
                                                                                    span { class: "rounded-md bg-muted/80 px-2 py-0.5 text-[10px] font-medium text-muted-foreground mr-1",
                                                                                        "{category}"
                                                                                    }
                                                                                    for &target_col in COLUMN_NAMES {
                                                                                        if target_col != column {
                                                                                            {
                                                                                                let t_id = id.clone();
                                                                                                let t_target = target_col.to_string();
                                                                                                let short_label = match target_col {
                                                                                                    "Pending" => "Pend",
                                                                                                    "InProgress" => "InProg",
                                                                                                    "Onhold" => "Hold",
                                                                                                    _ => "Done",
                                                                                                };
                                                                                                let move_title = format!("Change stage to {}", column_display_label(target_col));
                                                                                                rsx! {
                                                                                                    button {
                                                                                                        r#type: "button",
                                                                                                        key: "mv-{id}-{target_col}",
                                                                                                        title: "{move_title}",
                                                                                                        class: "cursor-pointer rounded px-1.5 py-0.5 text-[9px] font-semibold text-muted-foreground border border-border/60 hover:text-foreground hover:border-primary/50 hover:bg-primary/10 transition-colors",
                                                                                                        onclick: move |e: MouseEvent| {
                                                                                                            e.stop_propagation();
                                                                                                            move_to_column_end(t_id.clone(), t_target.clone());
                                                                                                        },
                                                                                                        "→ {short_label}"
                                                                                                    }
                                                                                                }
                                                                                            }
                                                                                        }
                                                                                    }
                                                                                }
                                                                                div {
                                                                                    title: "{ticket.assignee_name()} ({assignee})",
                                                                                    class: "flex size-5.5 shrink-0 items-center justify-center rounded-full text-[10px] font-bold bg-muted text-foreground border border-border/80 pointer-events-auto cursor-help",
                                                                                    "{assignee}"
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }

                                                    // Drop zone at bottom of column
                                                    div {
                                                        class: if is_col_over {
                                                            "flex items-center justify-center rounded-md border-2 border-dashed border-primary bg-primary/10 py-2.5 text-xs font-semibold text-primary transition-all animate-pulse"
                                                        } else {
                                                            "flex items-center justify-center rounded-md border border-dashed border-border/40 py-2 text-[11px] text-muted-foreground/50 transition-all hover:border-border hover:text-muted-foreground hover:bg-accent/20 cursor-pointer"
                                                        },
                                                        onclick: {
                                                            let col_c = column.to_string();
                                                            move |_| quick_add_ticket(col_c.clone())
                                                        },
                                                        ondragenter: move |e: DragEvent| {
                                                            e.prevent_default();
                                                            if drag_over_col.read().as_deref() != Some(column) {
                                                                drag_over_col.set(Some(column.to_string()));
                                                            }
                                                            drag_over_card.set(None);
                                                        },
                                                        ondragover: move |e: DragEvent| {
                                                            e.prevent_default();
                                                            e.stop_propagation();
                                                            if drag_over_col.read().as_deref() != Some(column) {
                                                                drag_over_col.set(Some(column.to_string()));
                                                            }
                                                            drag_over_card.set(None);
                                                        },
                                                        ondrop: move |e: DragEvent| {
                                                            e.prevent_default();
                                                            e.stop_propagation();
                                                            let src = e.data().data_transfer().get_as_text()
                                                                .filter(|s| !s.is_empty())
                                                                .or_else(|| dragged_id.read().clone());
                                                            if let Some(src) = src {
                                                                move_to_column_end(src, column.to_string());
                                                            }
                                                            clear_drag_state();
                                                        },
                                                        if is_col_over {
                                                            span { "Drop ticket here ({col_display})" }
                                                        } else {
                                                            span { "+ Quick Add Ticket ({col_display})" }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                // --------------------------------------------------------
                // IT Data Table View (Full Width, Sortable, Filterable, Paged)
                // --------------------------------------------------------
                TabsContent {
                    value: "table".to_string(),
                    class: "flex flex-col flex-1 min-h-0 w-full overflow-hidden pt-2",
                    DataTable {
                        data: tickets.read().clone(),
                        columns: ticket_columns(),
                        row_id: ticket_row_id,
                        search_text: ticket_search_text,
                        filter_label: "Status".to_string(),
                        filter: vec![
                            DataTableFilter::new("Pending", "Pending"),
                            DataTableFilter::new("InProgress", "In Progress"),
                            DataTableFilter::new("Onhold", "On Hold"),
                            DataTableFilter::new("Completed", "Completed"),
                        ],
                        filter_value: ticket_filter_value as fn(&TicketItem) -> String,
                        filter2_label: "Priority".to_string(),
                        filter2: vec![
                            DataTableFilter::new("Urgent", "Urgent"),
                            DataTableFilter::new("High", "High"),
                            DataTableFilter::new("Medium", "Medium"),
                            DataTableFilter::new("Low", "Low"),
                        ],
                        filter2_value: ticket_priority_filter_value as fn(&TicketItem) -> String,
                        on_row_click: move |t: TicketItem| state.open_view(t),
                        on_row_context_menu: move |(t, coords): (TicketItem, (f64, f64))| {
                            table_ctx_ticket.set(Some(t));
                            table_ctx_x.set(coords.0);
                            table_ctx_y.set(coords.1);
                            table_ctx_open.set(true);
                        },
                        page_size: 15,
                        class: "w-full flex-1 min-h-0".to_string(),
                    }
                }
            }

            // ============================================================
            // View Ticket Dialog Modal
            // ============================================================
            // ============================================================
            // View Ticket Dialog Modal (Clean & Focused on Essential Fields)
            // ============================================================
            Dialog {
                open: is_view_open,
                DialogContent {
                    class: "max-w-4xl sm:max-w-4xl w-full rounded-md border border-border/80 bg-card p-6 sm:p-7 shadow-2xl max-h-[88vh] overflow-y-auto",
                    if let Some(ticket) = viewing_ticket.read().as_ref() {
                        {
                            let t_for_edit = ticket.clone();
                            let status_lbl = column_display_label(&ticket.kanban_state);
                            let status_badge_cls = status_badge_class(&ticket.kanban_state);
                            let prio_variant = priority_badge(&ticket.priority);
                            let sla_cls = sla_badge_class(&ticket.sla_status);
                            let initial_letter = ticket.emp_name().chars().next().unwrap_or('U');

                            rsx! {
                                DialogHeader {
                                    div { class: "flex flex-wrap items-center justify-between gap-2 border-b border-border/50 pb-3",
                                        div { class: "flex items-center gap-2",
                                            span { class: "font-mono text-xs font-bold text-primary bg-primary/10 px-2.5 py-0.5 rounded",
                                                "{ticket.id}"
                                            }
                                            Badge {
                                                variant: "{prio_variant}",
                                                class: "text-[10px] font-semibold px-2 py-0.5",
                                                "{ticket.priority}"
                                            }
                                            Badge {
                                                variant: "{status_badge_variant(&ticket.kanban_state)}",
                                                class: "text-[10px] font-semibold {status_badge_cls}",
                                                "{status_lbl}"
                                            }
                                        }
                                        if !ticket.sla_status.is_empty() {
                                            span { class: "rounded px-2 py-0.5 text-[10px] font-semibold border {sla_cls}",
                                                "SLA: {ticket.sla_status}"
                                            }
                                        }
                                    }
                                    DialogTitle { class: "text-lg font-bold text-foreground mt-3 leading-snug",
                                        "{ticket.subject}"
                                    }
                                    DialogDescription { class: "text-xs text-muted-foreground mt-0.5",
                                        "Reported on {ticket.created_on} · Last updated: {ticket.last_updated_on}"
                                    }
                                }

                                div { class: "flex flex-col gap-4 py-3 text-xs",
                                    // 1. Created By (Requester Identity)
                                    div { class: "flex flex-col gap-1.5 rounded-md bg-muted/30 border border-border/40 p-3",
                                        span { class: "text-[10px] font-semibold text-muted-foreground uppercase tracking-wider", "Created By (Requester)" }
                                        div { class: "flex items-center justify-between gap-2 flex-wrap",
                                            div { class: "flex items-center gap-2.5",
                                                div { class: "flex size-8 shrink-0 items-center justify-center rounded-full bg-primary/10 text-primary font-bold text-xs border border-primary/20",
                                                    "{initial_letter}"
                                                }
                                                div { class: "flex flex-col min-w-0",
                                                    span { class: "font-semibold text-foreground text-xs", "{ticket.emp_name()}" }
                                                    span { class: "font-mono text-[11px] text-muted-foreground", "{ticket.emp_code_only()}" }
                                                }
                                            }
                                            div { class: "flex items-center gap-1.5 text-xs text-muted-foreground",
                                                span { class: "font-mono text-foreground/80 bg-background/60 px-2 py-0.5 rounded border border-border/50", "{ticket.email_address}" }
                                            }
                                        }
                                    }

                                    // 2. Assignment, Stage & Target Deadline
                                    div { class: "grid grid-cols-2 sm:grid-cols-4 gap-2.5",
                                        div { class: "flex flex-col gap-0.5 rounded-md bg-muted/20 p-2.5 border border-border/40",
                                            span { class: "text-[9px] font-semibold text-muted-foreground uppercase tracking-wider", "Assigned To" }
                                            span { class: "font-semibold text-foreground text-xs truncate", "{ticket.assignee_name()}" }
                                            span { class: "text-[10px] text-muted-foreground font-mono", "({ticket.assigned_to})" }
                                        }
                                        div { class: "flex flex-col gap-0.5 rounded-md bg-muted/20 p-2.5 border border-border/40",
                                            span { class: "text-[9px] font-semibold text-muted-foreground uppercase tracking-wider", "Stage" }
                                            span { class: "font-semibold text-foreground text-xs", "{status_lbl}" }
                                            span { class: "text-[10px] text-muted-foreground", "Kanban Stage" }
                                        }
                                        div { class: "flex flex-col gap-0.5 rounded-md bg-muted/20 p-2.5 border border-border/40",
                                            span { class: "text-[9px] font-semibold text-muted-foreground uppercase tracking-wider", "Category" }
                                            span { class: "font-semibold text-foreground text-xs", "{ticket.category}" }
                                            span { class: "text-[10px] text-muted-foreground", "IT Support" }
                                        }
                                        div { class: "flex flex-col gap-0.5 rounded-md bg-muted/20 p-2.5 border border-border/40",
                                            span { class: "text-[9px] font-semibold text-muted-foreground uppercase tracking-wider", "Target Deadline" }
                                            span { class: "font-mono font-semibold text-foreground text-xs truncate",
                                                if ticket.deadline.is_empty() { "None set" } else { "{ticket.deadline}" }
                                            }
                                            span { class: "text-[10px] text-muted-foreground", "Resolution Target" }
                                        }
                                    }

                                    // 3. Problem Description
                                    div { class: "flex flex-col gap-1.5",
                                        span { class: "text-[10px] font-semibold text-muted-foreground uppercase tracking-wider", "Problem Description" }
                                        div { class: "rounded-md bg-muted/40 p-3 text-xs leading-relaxed text-foreground border border-border/40 font-mono whitespace-pre-wrap",
                                            if ticket.problem_description.is_empty() {
                                                "No detailed description provided."
                                            } else {
                                                "{ticket.problem_description}"
                                            }
                                        }
                                    }

                                    // 4. Attachment (if present)
                                    if !ticket.attachment_name.is_empty() || !ticket.attachment_url.is_empty() {
                                        div { class: "flex items-center justify-between rounded-md bg-muted/30 border border-border/40 p-2.5",
                                            div { class: "flex items-center gap-2 min-w-0",
                                                span { class: "text-[10px] font-semibold text-muted-foreground uppercase", "Attachment:" }
                                                span { class: "font-mono font-medium text-foreground truncate", "{ticket.attachment_name}" }
                                            }
                                            if !ticket.attachment_url.is_empty() {
                                                span { class: "text-[11px] font-mono text-primary truncate max-w-[220px]", "{ticket.attachment_url}" }
                                            }
                                        }
                                    }
                                }

                                DialogFooter { class: "flex items-center justify-between gap-2 pt-3 border-t border-border/50",
                                    Button {
                                        variant: "outline",
                                        size: "sm",
                                        onclick: move |_| is_view_open.set(false),
                                        "Close"
                                    }
                                    Button {
                                        variant: "default",
                                        size: "sm",
                                        class: "gap-1.5",
                                        onclick: move |_| state.open_edit(t_for_edit.clone()),
                                        PencilIcon { class: "size-3.5" }
                                        "Modify Ticket"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // ============================================================
            // Modify Ticket Dialog Modal (Focused & Clean Form)
            // ============================================================
            Dialog {
                open: is_edit_open,
                DialogContent {
                    class: "max-w-4xl sm:max-w-4xl w-full rounded-md border border-border/80 bg-card p-6 sm:p-7 shadow-2xl max-h-[88vh] overflow-y-auto",
                    if let Some(ticket) = editing_ticket.read().as_ref() {
                        {
                            let t_id = ticket.id.clone();
                            rsx! {
                                DialogHeader {
                                    div { class: "flex items-center gap-2",
                                        span { class: "font-mono text-xs font-bold text-primary bg-primary/10 px-2.5 py-0.5 rounded",
                                            "{t_id}"
                                        }
                                        DialogTitle { class: "text-lg font-bold text-foreground",
                                            "Modify Ticket"
                                        }
                                    }
                                    DialogDescription { class: "text-xs text-muted-foreground mt-0.5",
                                        "Update ticket details, requester information, assignment, and stage."
                                    }
                                }

                                div { class: "flex flex-col gap-4 py-3 text-xs",
                                    // 1. Subject
                                    div { class: "flex flex-col gap-1",
                                        label { class: "text-xs font-semibold text-foreground", "Subject" }
                                        Input {
                                            class: "text-xs h-9 w-full",
                                            placeholder: "Enter ticket subject...".to_string(),
                                            value: edit_subject.read().clone(),
                                            oninput: move |e: FormEvent| edit_subject.set(e.value()),
                                        }
                                    }

                                    // 2. Created By (Requester Details)
                                    div { class: "rounded-md border border-border/60 bg-muted/20 p-3 flex flex-col gap-2.5",
                                        span { class: "text-[10px] font-semibold text-muted-foreground uppercase tracking-wider", "Created By (Requester)" }
                                        div { class: "grid grid-cols-1 sm:grid-cols-3 gap-2.5",
                                            div { class: "flex flex-col gap-1",
                                                label { class: "text-[11px] font-semibold text-foreground", "Employee Name" }
                                                Input {
                                                    class: "text-xs h-9 w-full",
                                                    placeholder: "e.g. David Chen".to_string(),
                                                    value: edit_emp_name.read().clone(),
                                                    oninput: move |e: FormEvent| edit_emp_name.set(e.value()),
                                                }
                                            }
                                            div { class: "flex flex-col gap-1",
                                                label { class: "text-[11px] font-semibold text-foreground", "Employee Code" }
                                                Input {
                                                    class: "text-xs h-9 w-full",
                                                    placeholder: "e.g. EMP-4019".to_string(),
                                                    value: edit_employee_code.read().clone(),
                                                    oninput: move |e: FormEvent| edit_employee_code.set(e.value()),
                                                }
                                            }
                                            div { class: "flex flex-col gap-1",
                                                label { class: "text-[11px] font-semibold text-foreground", "Email Address" }
                                                Input {
                                                    class: "text-xs h-9 w-full",
                                                    placeholder: "e.g. name@corp.internal".to_string(),
                                                    value: edit_email_address.read().clone(),
                                                    oninput: move |e: FormEvent| edit_email_address.set(e.value()),
                                                }
                                            }
                                        }
                                    }

                                    // 3. Classification, Stage & Assignment
                                    div { class: "grid grid-cols-2 sm:grid-cols-4 gap-3",
                                        div { class: "flex flex-col gap-1",
                                            label { class: "text-[11px] font-semibold text-foreground", "Category" }
                                            select {
                                                class: "h-9 rounded-md border border-border/80 bg-card px-2 text-xs text-foreground focus:border-primary focus:outline-none cursor-pointer",
                                                value: "{edit_category}",
                                                onchange: move |e: FormEvent| edit_category.set(e.value()),
                                                option { value: "Network", "Network" }
                                                option { value: "Hardware", "Hardware" }
                                                option { value: "Access", "Access" }
                                                option { value: "Security", "Security" }
                                                option { value: "Software", "Software" }
                                                option { value: "Infrastructure", "Infrastructure" }
                                                option { value: "General", "General" }
                                            }
                                        }
                                        div { class: "flex flex-col gap-1",
                                            label { class: "text-[11px] font-semibold text-foreground", "Priority" }
                                            select {
                                                class: "h-9 rounded-md border border-border/80 bg-card px-2 text-xs text-foreground focus:border-primary focus:outline-none cursor-pointer",
                                                value: "{edit_priority}",
                                                onchange: move |e: FormEvent| edit_priority.set(e.value()),
                                                option { value: "High", "High" }
                                                option { value: "Medium", "Medium" }
                                                option { value: "Low", "Low" }
                                                option { value: "Critical", "Critical" }
                                            }
                                        }
                                        div { class: "flex flex-col gap-1",
                                            label { class: "text-[11px] font-semibold text-foreground", "Stage" }
                                            select {
                                                class: "h-9 rounded-md border border-border/80 bg-card px-2 text-xs text-foreground focus:border-primary focus:outline-none cursor-pointer",
                                                value: "{edit_kanban_state}",
                                                onchange: move |e: FormEvent| {
                                                    let val = e.value();
                                                    edit_kanban_state.set(val.clone());
                                                    edit_stage.set(val.clone());
                                                    edit_status.set(val);
                                                },
                                                option { value: "Pending", "Pending" }
                                                option { value: "InProgress", "In Progress" }
                                                option { value: "Onhold", "On Hold" }
                                                option { value: "Completed", "Completed" }
                                            }
                                        }
                                        div { class: "flex flex-col gap-1",
                                            label { class: "text-[11px] font-semibold text-foreground", "Assigned To" }
                                            select {
                                                class: "h-9 rounded-md border border-border/80 bg-card px-2 text-xs text-foreground focus:border-primary focus:outline-none cursor-pointer",
                                                value: "{edit_assigned_to}",
                                                onchange: move |e: FormEvent| edit_assigned_to.set(e.value()),
                                                option { value: "GR", "Gabriel Ross (GR)" }
                                                option { value: "AL", "Alex Lee (AL)" }
                                                option { value: "JD", "Jordan Davis (JD)" }
                                                option { value: "SL", "Sam Lin (SL)" }
                                            }
                                        }
                                    }

                                    // 4. Problem Description
                                    div { class: "flex flex-col gap-1",
                                        label { class: "text-[11px] font-semibold text-foreground", "Problem Description" }
                                        textarea {
                                            class: "min-h-[85px] rounded-md border border-border/80 bg-card p-2 text-xs text-foreground focus:border-primary focus:outline-none font-mono",
                                            placeholder: "Detailed description of the issue...",
                                            value: "{edit_problem_description}",
                                            oninput: move |e: FormEvent| edit_problem_description.set(e.value()),
                                        }
                                    }

                                    // 5. Target Deadline & SLA Status
                                    div { class: "grid grid-cols-1 sm:grid-cols-2 gap-3",
                                        div { class: "flex flex-col gap-1",
                                            label { class: "text-[11px] font-semibold text-foreground", "Target Deadline" }
                                            Input {
                                                class: "text-xs h-9 w-full",
                                                placeholder: "e.g. Today · 05:00 PM or 2026-11-05".to_string(),
                                                value: edit_deadline.read().clone(),
                                                oninput: move |e: FormEvent| edit_deadline.set(e.value()),
                                            }
                                        }
                                        div { class: "flex flex-col gap-1",
                                            label { class: "text-[11px] font-semibold text-foreground", "SLA Status" }
                                            select {
                                                class: "h-9 rounded-md border border-border/80 bg-card px-2 text-xs text-foreground focus:border-primary focus:outline-none cursor-pointer",
                                                value: "{edit_sla_status}",
                                                onchange: move |e: FormEvent| edit_sla_status.set(e.value()),
                                                option { value: "Within SLA", "Within SLA" }
                                                option { value: "Approaching Breach", "Approaching Breach" }
                                                option { value: "Breached", "Breached" }
                                                option { value: "Excluded", "Excluded" }
                                            }
                                        }
                                    }
                                }

                                DialogFooter { class: "flex items-center justify-end gap-2 pt-3 border-t border-border/50",
                                    Button {
                                        variant: "outline",
                                        size: "sm",
                                        onclick: move |_| {
                                            is_edit_open.set(false);
                                            editing_ticket.set(None);
                                        },
                                        "Cancel"
                                    }
                                    Button {
                                        variant: "default",
                                        size: "sm",
                                        onclick: move |_| save_edit(),
                                        "Save Changes"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if table_ctx_open() {
                if let Some(ticket) = table_ctx_ticket.read().clone() {
                    TableFloatingContextMenu {
                        ticket,
                        open: table_ctx_open,
                        x: table_ctx_x,
                        y: table_ctx_y,
                        on_view: move |t| state.open_view(t),
                        on_edit: move |t| state.open_edit(t),
                    }
                }
            }
        }
    }
}

