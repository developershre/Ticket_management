# BPSL IT Ticket Management System (TMS)

[![Rust](https://img.shields.io/badge/Rust-1.85%2B-orange.svg?style=flat&logo=rust)](https://www.rust-lang.org/)
[![Dioxus](https://img.shields.io/badge/Dioxus-0.7-blue.svg?style=flat&logo=rust)](https://dioxuslabs.com/)
[![SQLite](https://img.shields.io/badge/SQLite-3-003B57.svg?style=flat&logo=sqlite)](https://www.sqlite.org/)
[![Tailwind CSS](https://img.shields.io/badge/Tailwind_CSS-v4-38B2AC.svg?style=flat&logo=tailwind-css)](https://tailwindcss.com/)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20Linux-brightgreen.svg?style=flat)](https://github.com/)

A high-performance, offline-first desktop Ticket & Task Management System built in **Rust** using **Dioxus 0.7**, backed by an embedded **SQLite** database and synchronized bidirectionally with **Google Sheets** (via Google Apps Script Web App) and **BPSL REST API**.

---

## 🌟 Key Features

### 📋 Multi-State Kanban Board
- **Fluid Drag-and-Drop**: Move cards between workflow columns with instant visual feedback.
- **Granular Multi-State Tracking**: Full support for distinct ticket lifecycles (`Pending`, `InProgress`, `Hold`, `Solved`, `Completed`, and `Cancelled`).
- **Custom Views & Filters**: Create and pin custom view tabs, set Work-In-Progress (WIP) column limits, and hide empty columns.
- **Context Menus**: Floating context menus for fast status changes, priority reassignment, solver assignment, and ticket modifications.
- **Action Badging**: Auto-extracted activity badges (e.g., `WB`, `SALE ORDER`, `AMD`) synced directly with sheet stage columns.

### 📊 Data Table View
- **High-Velocity Grid**: Search and paginate across thousands of records with zero latency.
- **Advanced Filtering**: Filter by lifecycle status (Open vs. Closed), ticket priority, category, assignee, date preset (Today, Yesterday, This Week, This Month), SLA status, and attachment presence.
- **Multi-Field Search**: Real-time searching across ticket IDs, subjects, descriptions, employee names/codes, departments, and tags.

### 🔄 Intelligent Bidirectional Sync Engine
- **Background Worker**: Continuous background synchronization (20-second cadence + reactive instant wake upon local changes).
- **Offline Push Queue**: Queued local changes persist across app restarts and coalesce automatically before dispatching.
- **Automated Conflict Resolution**: Detects concurrent remote edits, tracks field-level diffs (`sheet_changes`), and preserves local edits during connection drops.
- **Dual Wire Protocols**: Supports both direct Google Apps Script Web App envelopes (`LegacyV1`) and BPSL REST APIs (`BpslRest`).

### 💾 Zero-Setup Embedded SQLite
- **ACID Transactional Storage**: Local single-file database (`management.db`) requiring no external servers or drivers.
- **Prone-Free Migrations**: Dynamic schema verification and auto-repair on startup with CLI migration tools.
- **Clean Bundling**: Automated empty database seeding tools (`--init-clean-db`, `--bundle-prep`) for distribution packages.

### 📈 Analytics & KPI Dashboard
- **Real-Time Metrics**: Total hours logged, open/closed counts, and average resolution times.
- **Visual Charts**: SLA compliance meters, category distribution bar charts, department breakdown graphs, and weekly activity charts.
- **Solver Leaderboard**: Track agent workload, resolution rates, and customer satisfaction ratings.

### 🔔 Reminders, Notes & Calendar
- **Interactive Reminders**: Schedule follow-ups with flexible 12-hour/24-hour time pickers and repeat intervals.
- **Native Desktop Notifications**: Windows toast and cross-platform desktop notifications for new tickets and reminder alerts.
- **Scratchpad Notes**: Offline personal notepad with instant local persistence and quick search.
- **Calendar View**: Monthly calendar displaying scheduled ticket deadlines and reminder timestamps.

### 🎨 Fast Theme Engine
- **Zero-Latency Toggle**: Instant switching between Dark, Light, and System themes without transition flashes or page reload.

---

## 🏗️ Project Architecture

The codebase is organized cleanly into modular, decoupled domains:

```text
Ticket_management/
├── assets/                     # Static UI assets (icons, stylesheets, DnD scripts)
│   ├── favicon.ico             # Application icon
│   ├── logo.svg / logo.png     # Branding logos
│   ├── dnd.js                  # Native drag-and-drop integration
│   ├── main.css                # Base custom styles
│   └── tailwind.css            # Compiled Tailwind CSS v4 styles
├── docs/                       # Technical specifications and guides
│   ├── GOOGLE_SHEET_SPECIFICATION.md  # Standard Google Sheet tabs & schema
│   └── LINUX_PACKAGING.md      # Linux packaging guide (.rpm, .deb, .pkg.tar.zst)
├── scripts/                    # Automation and packaging utilities
│   ├── GoogleAppsScript_Universal_TMS.js  # Production Google Apps Script backend
│   ├── create_deb.js           # Cross-platform pure Node.js .deb generator
│   └── generate_empty_db.js    # Pristine empty database generator
├── src/                        # Rust application source code
│   ├── main.rs                 # App entry point, CLI handlers, window config, routes
│   ├── api.rs                  # Wire protocol client (Google Apps Script / BPSL REST)
│   ├── db.rs                   # SQLite database operations, queries, and migrations
│   ├── sync.rs                 # Sync engine, worker thread, queue & conflict manager
│   ├── sheet_crud.rs           # Generic SheetEntity CRUD abstractions
│   ├── notifications.rs        # Desktop toast notification dispatcher
│   ├── state.rs                # Shared reactive application state & time utilities
│   ├── icons.rs                # Scalable Vector Graphics (SVG) icon library
│   ├── components/             # Reusable UI primitives and shared widgets
│   │   ├── app_sidebar.rs      # Main navigation sidebar
│   │   ├── titlebar.rs         # Custom frameless window titlebar
│   │   ├── fab.rs              # Quick-action floating action button
│   │   ├── settings_modal.rs   # API connection & sheet configuration modal
│   │   ├── sheet_changes_modal.rs # Remote change audit log viewer
│   │   └── ui/                 # Atomic UI components (button, dialog, tabs, etc.)
│   └── pages/                  # Top-level application views
│       ├── tickets/            # Kanban board, data table, dialogs, filters
│       ├── analytics/          # KPI cards, charts, solver leaderboard
│       ├── calendar_page.rs    # Interactive monthly calendar
│       ├── employees.rs        # Employee reference directory
│       ├── notes.rs            # Scratchpad notes page
│       └── reminders.rs        # Follow-up reminders management
├── build-linux.sh / .ps1       # Automated Linux packaging scripts
├── build-msi.ps1               # Automated Windows MSI packaging script
├── Cargo.toml                  # Rust dependencies and feature flags
├── Dioxus.toml                 # Dioxus desktop bundle configuration
└── README.md                   # Project documentation
```

---

## 🚀 Getting Started

### Prerequisites

1. **Rust**: Install the latest stable toolchain via [rustup.rs](https://rustup.rs/):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
2. **Dioxus CLI**: Install the `dx` command line utility:
   ```bash
   cargo install dioxus-cli --version 0.7.1
   ```
3. **Node.js** (optional, for Tailwind CSS compilation):
   ```bash
   npm install
   ```

### Running the Application

```bash
# Run in development mode (Desktop)
dx serve --platform desktop

# Or run directly via Cargo
cargo run
```

### Running Tests

Run the test suite with single-threaded database isolation:

```bash
cargo test -- --test-threads=1
```

---

## 🛠️ Command-Line Utilities

The executable provides built-in operational and maintenance commands:

| Command | Description |
|---|---|
| `management --init-clean-db [path]` | Generates a single pristine empty SQLite database with all tables (0 rows, zero pre-entered settings). |
| `management --migrate-db` | Verifies and updates the active SQLite database schema safely without erasing user data. |
| `management --reset-db` | Clears all rows from the active local database and runs `VACUUM`. |
| `management --sync-once` | Executes a single blocking sync cycle against the configured API and prints diagnostics without launching the GUI. |
| `management --sync-test` | Runs the automated offline queue and reconciliation test harness. |

---

## 📦 Production Packaging

### Windows MSI Installer

Use the automated PowerShell script to compile and generate a signed/standard `.msi` using the WiX Toolset:

```powershell
.\build-msi.ps1
```

The output MSI installer is placed in `target/dx/management/release/windows/`.

### Linux Packages (.rpm, .deb, .pkg.tar.zst, .tar.gz)

Use the automated Linux build script:

```bash
# Build installers for all supported distributions
./build-linux.sh --all

# Or build for a specific distribution:
./build-linux.sh --fedora      # Fedora / RHEL / CentOS (.rpm)
./build-linux.sh --mint        # Linux Mint / Ubuntu / Debian (.deb)
./build-linux.sh --arch        # Arch Linux / Manjaro (.pkg.tar.zst + PKGBUILD)
./build-linux.sh --tarball     # Universal portable archive (.tar.gz)
```

For complete distribution details and dependencies, refer to [docs/LINUX_PACKAGING.md](docs/LINUX_PACKAGING.md).

---

## 📖 Additional Documentation

- [Google Sheet Universal Specification](docs/GOOGLE_SHEET_SPECIFICATION.md): Defines the 4 standard Google Sheet tabs, data validation choices, and Universal CRUD API contract.
- [Linux Packaging Guide](docs/LINUX_PACKAGING.md): Complete instructions for installing, building, and deploying Linux packages across various distributions.

---

## 📄 License

Internal proprietary software developed for BPSL IT Support. All rights reserved.
