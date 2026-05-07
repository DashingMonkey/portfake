# Portfake

A lightweight API Mock Server built with Tauri + Vue 3 + Rust.

## Features

- **Mock API Management**: Create and organize mock API endpoints into collections
- **Local Mock Server**: Run a local server to serve your mock APIs
- **Request Editor**: Define method, URL, headers, and response body for each endpoint
- **Response Templates**: Use template syntax to generate dynamic responses
- **Collections**: Organize mock endpoints into folders with drag-and-drop
- **Import/Export**: Share collections easily
- **VSCode-inspired UI**: Light/dark themes, resizable panels, tabbed interface

## Tech Stack

| Layer | Technology |
|-------|------------|
| Desktop Framework | Tauri 2 |
| Frontend | Vue 3 + TypeScript |
| Styling | Tailwind CSS |
| State Management | Pinia |
| HTTP Server | Axum (Rust) |
| Database | SQLite (rusqlite) |

## Architecture

```
portfake/
├── src/                          # Vue frontend
│   ├── components/               # UI components
│   │   ├── common/              # EmptyState, SettingsDialog, Toast, MethodSelect
│   │   ├── layout/             # AppHeader
│   │   ├── tabs/               # TabBar
│   │   └── workspace/          # SidebarPanel, MainWorkspace, RequestEditor
│   ├── stores/                  # Pinia stores
│   │   ├── tabs.ts            # Tab management
│   │   ├── settings.ts        # Theme, UI state
│   │   ├── server.ts          # Mock server state
│   │   ├── collections.ts     # Collection tree management
│   │   └── types.ts           # TypeScript types
│   └── composables/             # Vue composables
│       └── useToast.ts         # Toast notifications
│
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── commands/           # Tauri command handlers
│   │   │   ├── server.rs      # Mock server control (start/stop)
│   │   │   ├── collections.rs # Collection CRUD
│   │   │   ├── requests.rs    # Request CRUD
│   │   │   └── examples.rs    # Example generation
│   │   ├── db/                # Database layer
│   │   │   ├── mod.rs         # SQLite connection
│   │   │   ├── schema.rs      # Table definitions
│   │   │   ├── collections.rs # Collection operations
│   │   │   ├── requests.rs    # Request operations
│   │   │   └── examples.rs    # Example operations
│   │   ├── models/            # Data models
│   │   ├── server/            # Axum HTTP server
│   │   │   └── mod.rs         # Mock server implementation
│   │   ├── template/          # Response template engine
│   │   │   └── mod.rs         # Template parsing & rendering
│   │   └── example_gen/       # Example response generator
│   │       └── mod.rs
│   └── Cargo.toml
│
├── package.json                  # Node dependencies
├── vite.config.ts               # Vite bundler config
├── tailwind.config.js           # Tailwind theme config
└── tauri.conf.json             # Tauri app config
```

## Data Storage

### SQLite Database (Per Workspace)

Each workspace is a separate `.db` file located in the executable directory.

| Table | Description |
|-------|-------------|
| `collections` | Folder hierarchy (id, parent_id, name, sort_order) |
| `requests` | Mock endpoints (method, path, headers, body, response_delay) |
| `examples` | Example responses linked to requests |

### Browser localStorage (Per Workspace)

| Key | Description |
|-----|-------------|
| `portfake-{workspace}-tabs` | Open tabs state |
| `portfake-{workspace}-drafts` | Unsaved request changes |
| `portfake-settings` | Global settings (theme, active workspace, UI state) |

## Dependencies

### Frontend (package.json)

```
vue@^3.5.32
pinia@^3.0.4
@headlessui/vue
@tauri-apps/api@^2.0.0
@tauri-apps/cli@^2.0.0
@vitejs/plugin-vue@^5.0.0
tailwindcss
typescript
vite
vue-tsc
```

### Backend (Cargo.toml)

```
tauri@2
tauri-plugin-shell
serde/serde_json
tokio (full)
rusqlite (bundled)
uuid
chrono
axum
tower
tower-http (cors)
log/env_logger
anyhow
rand
regex
```

## Build

```bash
# Install frontend dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

## Mock Server

Portfake runs a local Axum-based HTTP server to serve your mock APIs:

- Default address: `http://localhost:3210`
- Serves all configured mock endpoints
- Supports dynamic response templates
- Configurable response delay for testing

## Window & UI

- Default size: 1280x800, minimum: 900x600
- Custom frameless window with title bar controls
- VSCode-inspired activity bar and sidebar layout
- Light/dark theme support
