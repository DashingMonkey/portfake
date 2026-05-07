use rusqlite::{Connection, Result};

pub fn run(conn: &Connection) -> Result<()> {
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    conn.execute(CREATE_COLLECTIONS_TABLE, [])?;
    conn.execute(CREATE_REQUESTS_TABLE, [])?;
    conn.execute(CREATE_EXAMPLES_TABLE, [])?;
    conn.execute(CREATE_SERVER_CONFIG_TABLE, [])?;

    conn.execute(INIT_SERVER_CONFIG, [])?;

    Ok(())
}

const CREATE_COLLECTIONS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS collections (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    source_workspace TEXT NOT NULL,
    source_collection_id TEXT,
    created_at TEXT NOT NULL,
    enabled INTEGER DEFAULT 1
)
"#;

const CREATE_REQUESTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS requests (
    id TEXT PRIMARY KEY,
    collection_id TEXT NOT NULL,
    name TEXT NOT NULL,
    method TEXT NOT NULL,
    path TEXT NOT NULL,
    description TEXT,
    headers TEXT DEFAULT '[]',
    position INTEGER DEFAULT 0,
    enabled INTEGER DEFAULT 1,
    FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE CASCADE
)
"#;

const CREATE_EXAMPLES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS examples (
    id TEXT PRIMARY KEY,
    request_id TEXT NOT NULL,
    name TEXT NOT NULL,
    is_default INTEGER DEFAULT 1,
    status_code INTEGER NOT NULL DEFAULT 200,
    headers TEXT DEFAULT '[]',
    body TEXT DEFAULT '',
    body_type TEXT DEFAULT 'json',
    delay_ms INTEGER,
    match_rules TEXT DEFAULT '[]',
    order_index INTEGER DEFAULT 0,
    FOREIGN KEY (request_id) REFERENCES requests(id) ON DELETE CASCADE
)
"#;

const CREATE_SERVER_CONFIG_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS server_config (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    port INTEGER DEFAULT 3210,
    enabled INTEGER DEFAULT 0,
    cors_origins TEXT DEFAULT '["*"]'
)
"#;

const INIT_SERVER_CONFIG: &str = r#"
INSERT OR IGNORE INTO server_config (id, port, enabled, cors_origins) VALUES (1, 3210, 0, '["*"]')
"#;
