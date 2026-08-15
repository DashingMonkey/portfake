use crate::db::Database;
use crate::models::Example;
use rusqlite::params;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct CreateExampleParams {
    pub db: Arc<Database>,
    pub id: String,
    pub request_id: String,
    pub name: String,
    pub is_default: bool,
    pub status_code: u16,
    pub headers: String,
    pub body: String,
    pub body_type: String,
    pub delay_ms: Option<i64>,
}

pub fn get_examples_by_request(
    db: &Arc<Database>,
    request_id: &str,
) -> Result<Vec<Example>, String> {
    let conn = db.connection().lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, request_id, name, is_default, status_code, headers, body, body_type, delay_ms, match_rules, order_index FROM examples WHERE request_id = ?1 ORDER BY order_index ASC",
        )
        .map_err(|e| e.to_string())?;

    let examples = stmt
        .query_map([request_id], |row| {
            Ok(Example {
                id: row.get(0)?,
                request_id: row.get(1)?,
                name: row.get(2)?,
                is_default: row.get::<_, i32>(3)? != 0,
                status_code: row.get::<_, i32>(4)? as u16,
                headers: row.get(5)?,
                body: row.get(6)?,
                body_type: row.get(7)?,
                delay_ms: row.get(8)?,
                match_rules: row.get(9)?,
                order_index: row.get(10)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(examples)
}

pub fn create_example(params: CreateExampleParams) -> Result<Example, String> {
    let conn = params.db.connection().lock().map_err(|e| e.to_string())?;

    let max_idx: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(order_index), -1) + 1 FROM examples WHERE request_id = ?1",
            [params.request_id.as_str()],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO examples (id, request_id, name, is_default, status_code, headers, body, body_type, delay_ms, match_rules, order_index) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            params.id,
            params.request_id,
            params.name,
            params.is_default as i32,
            params.status_code as i32,
            params.headers,
            params.body,
            params.body_type,
            params.delay_ms,
            "[]",
            max_idx
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(Example {
        id: params.id,
        request_id: params.request_id,
        name: params.name,
        is_default: params.is_default,
        status_code: params.status_code,
        headers: params.headers,
        body: params.body,
        body_type: params.body_type,
        delay_ms: params.delay_ms,
        match_rules: "[]".to_string(),
        order_index: max_idx,
    })
}

pub fn update_example(
    db: &Arc<Database>,
    example_id: &str,
    status_code: u16,
    headers: &str,
    body: &str,
    body_type: &str,
    delay_ms: Option<i64>,
) -> Result<(), String> {
    let conn = db.connection().lock().map_err(|e| e.to_string())?;
    let affected = conn.execute(
        "UPDATE examples SET status_code = ?1, headers = ?2, body = ?3, body_type = ?4, delay_ms = ?5 WHERE id = ?6",
        params![status_code as i32, headers, body, body_type, delay_ms, example_id],
    ).map_err(|e| e.to_string())?;
    if affected == 0 {
        return Err(format!("Example with id '{}' not found", example_id));
    }
    Ok(())
}
