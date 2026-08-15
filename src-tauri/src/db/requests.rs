use crate::db::Database;
use crate::models::Request;
use rusqlite::params;
use std::sync::Arc;

pub fn get_requests_by_collection(
    db: &Arc<Database>,
    collection_id: &str,
) -> Result<Vec<Request>, String> {
    let conn = db.connection().lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, collection_id, name, method, path, description, headers, position, enabled FROM requests WHERE collection_id = ?1 ORDER BY name ASC",
        )
        .map_err(|e| e.to_string())?;

    let requests = stmt
        .query_map([collection_id], |row| {
            Ok(Request {
                id: row.get(0)?,
                collection_id: row.get(1)?,
                name: row.get(2)?,
                method: row.get(3)?,
                path: row.get(4)?,
                description: row.get(5)?,
                headers: row.get(6)?,
                position: row.get(7)?,
                enabled: row.get::<_, i32>(8)? != 0,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(requests)
}

pub fn create_request(
    db: &Arc<Database>,
    id: &str,
    collection_id: &str,
    name: &str,
    method: &str,
    path: &str,
    headers: &str,
) -> Result<Request, String> {
    let conn = db.connection().lock().map_err(|e| e.to_string())?;

    let max_pos: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(position), -1) + 1 FROM requests WHERE collection_id = ?1",
            [collection_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO requests (id, collection_id, name, method, path, headers, position, enabled) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1)",
        params![id, collection_id, name, method, path, headers, max_pos],
    )
    .map_err(|e| e.to_string())?;

    Ok(Request {
        id: id.to_string(),
        collection_id: collection_id.to_string(),
        name: name.to_string(),
        method: method.to_string(),
        path: path.to_string(),
        description: None,
        headers: headers.to_string(),
        position: max_pos,
        enabled: true,
    })
}

pub fn delete_request(db: &Arc<Database>, request_id: &str) -> Result<(), String> {
    let conn = db.connection().lock().map_err(|e| e.to_string())?;
    conn.execute(
        "DELETE FROM requests WHERE id = ?1",
        [request_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn update_request(
    db: &Arc<Database>,
    request_id: &str,
    collection_id: &str,
    name: &str,
    method: &str,
    path: &str,
    headers: &str,
) -> Result<(), String> {
    let conn = db.connection().lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE requests SET collection_id = ?1, name = ?2, method = ?3, path = ?4, headers = ?5 WHERE id = ?6",
        params![collection_id, name, method, path, headers, request_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn check_duplicate_request(
    db: &Arc<Database>,
    method: &str,
    path: &str,
    exclude_id: Option<&str>,
) -> Result<bool, String> {
    let conn = db.connection().lock().map_err(|e| e.to_string())?;
    let sql = match exclude_id {
        Some(_) => "SELECT COUNT(*) FROM requests WHERE method = ?1 AND path = ?2 AND id != ?3",
        None => "SELECT COUNT(*) FROM requests WHERE method = ?1 AND path = ?2",
    };
    let count: i64 = match exclude_id {
        Some(id) => conn.query_row(sql, params![method, path, id], |row| row.get(0)),
        None => conn.query_row(sql, [method, path], |row| row.get(0)),
    }
    .map_err(|e| e.to_string())?;
    Ok(count > 0)
}

pub fn set_request_enabled(db: &Arc<Database>, request_id: &str, enabled: bool) -> Result<(), String> {
    let conn = db.connection().lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE requests SET enabled = ?1 WHERE id = ?2",
        params![if enabled { 1 } else { 0 }, request_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
