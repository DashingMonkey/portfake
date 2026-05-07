use crate::db::Database;
use crate::models::Collection;
use rusqlite::params;
use std::sync::Arc;

pub fn get_collections(db: &Arc<Database>) -> Result<Vec<Collection>, String> {
    let conn = db.connection().lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, name, description, source_workspace, source_collection_id, created_at, enabled FROM collections ORDER BY name ASC")
        .map_err(|e| e.to_string())?;

    let collections = stmt
        .query_map([], |row| {
            Ok(Collection {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                source_workspace: row.get(3)?,
                source_collection_id: row.get(4)?,
                created_at: row.get(5)?,
                enabled: row.get::<_, i32>(6)? != 0,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(collections)
}

pub fn create_collection(
    db: &Arc<Database>,
    id: &str,
    name: &str,
    source_workspace: &str,
    source_collection_id: Option<&str>,
) -> Result<Collection, String> {
    let conn = db.connection().lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO collections (id, name, source_workspace, source_collection_id, created_at, enabled) VALUES (?1, ?2, ?3, ?4, ?5, 1)",
        params![id, name, source_workspace, source_collection_id, now],
    )
    .map_err(|e| e.to_string())?;

    Ok(Collection {
        id: id.to_string(),
        name: name.to_string(),
        description: None,
        source_workspace: source_workspace.to_string(),
        source_collection_id: source_collection_id.map(|s| s.to_string()),
        created_at: now,
        enabled: true,
    })
}

pub fn delete_collection(db: &Arc<Database>, id: &str) -> Result<(), String> {
    let conn = db.connection().lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM collections WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn rename_collection(db: &Arc<Database>, id: &str, name: &str) -> Result<Collection, String> {
    let conn = db.connection().lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE collections SET name = ?1 WHERE id = ?2",
        params![name, id],
    )
    .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, name, description, source_workspace, source_collection_id, created_at, enabled FROM collections WHERE id = ?1")
        .map_err(|e| e.to_string())?;

    let collection = stmt
        .query_row(params![id], |row| {
            Ok(Collection {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                source_workspace: row.get(3)?,
                source_collection_id: row.get(4)?,
                created_at: row.get(5)?,
                enabled: row.get::<_, i32>(6)? != 0,
            })
        })
        .map_err(|e| e.to_string())?;

    Ok(collection)
}

pub fn set_collection_enabled(db: &Arc<Database>, id: &str, enabled: bool) -> Result<(), String> {
    let conn = db.connection().lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE collections SET enabled = ?1 WHERE id = ?2",
        params![if enabled { 1 } else { 0 }, id],
    )
    .map_err(|e| e.to_string())?;

    // If disabling collection, also disable all its requests
    if !enabled {
        conn.execute(
            "UPDATE requests SET enabled = 0 WHERE collection_id = ?1",
            params![id],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}
