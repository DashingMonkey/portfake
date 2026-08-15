use crate::models::GenRequest;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GeneratedExample {
    pub name: String,
    pub status_code: u16,
    pub headers: String,
    pub body: String,
    pub body_type: String,
    pub delay_ms: Option<i64>,
}

pub fn generate_example(request: &GenRequest) -> GeneratedExample {
    let method = request.method.to_uppercase();
    let path = extract_path_segment(&request.url);

    match method.as_str() {
        "GET" => generate_get_example(&path),
        "POST" => generate_post_example(&path),
        "PUT" | "PATCH" => generate_put_example(),
        "DELETE" => generate_delete_example(),
        _ => generate_default_example(),
    }
}

fn generate_get_example(path: &str) -> GeneratedExample {
    let is_plural = path.ends_with('s') && path.len() > 1;
    let has_id_param = path.contains(":id") || path.matches('/').count() > 1 && !path.ends_with('s');

    let body = if has_id_param {
        // Single item
        let resource = extract_resource_name(path);
        generate_single_object(&resource)
    } else if is_plural {
        // Array of items
        let resource = extract_resource_name(path);
        format!(
            "[{},{},{},{},{}]",
            generate_single_object(&resource),
            generate_single_object(&resource),
            generate_single_object(&resource),
            generate_single_object(&resource),
            generate_single_object(&resource)
        )
    } else {
        // Single item by default
        let resource = extract_resource_name(path);
        generate_single_object(&resource)
    };

    GeneratedExample {
        name: "Default".to_string(),
        status_code: 200,
        headers: r#"[
  {
    "key": "Content-Type",
    "value": "application/json",
    "enabled": true
  }
]"#.to_string(),
        body,
        body_type: "json".to_string(),
        delay_ms: None,
    }
}

fn generate_post_example(_path: &str) -> GeneratedExample {
    GeneratedExample {
        name: "Created".to_string(),
        status_code: 201,
        headers: r#"[
  {
    "key": "Content-Type",
    "value": "application/json",
    "enabled": true
  }
]"#.to_string(),
        body: r#"{
  "success": true,
  "id": "{{$randomUuid}}",
  "createdAt": "{{$timestamp}}"
}"#.to_string(),
        body_type: "json".to_string(),
        delay_ms: None,
    }
}

fn generate_put_example() -> GeneratedExample {
    GeneratedExample {
        name: "Updated".to_string(),
        status_code: 200,
        headers: r#"[
  {
    "key": "Content-Type",
    "value": "application/json",
    "enabled": true
  }
]"#.to_string(),
        body: r#"{
  "success": true,
  "updatedAt": "{{$timestamp}}"
}"#.to_string(),
        body_type: "json".to_string(),
        delay_ms: None,
    }
}

fn generate_delete_example() -> GeneratedExample {
    GeneratedExample {
        name: "Deleted".to_string(),
        status_code: 204,
        headers: "[]".to_string(),
        body: "".to_string(),
        body_type: "text".to_string(),
        delay_ms: None,
    }
}

fn generate_default_example() -> GeneratedExample {
    GeneratedExample {
        name: "Default".to_string(),
        status_code: 200,
        headers: r#"[
  {
    "key": "Content-Type",
    "value": "application/json",
    "enabled": true
  }
]"#.to_string(),
        body: r#"{
  "success": true
}"#.to_string(),
        body_type: "json".to_string(),
        delay_ms: None,
    }
}

fn extract_path_segment(url: &str) -> String {
    if url.starts_with("http://") || url.starts_with("https://") {
        let after_scheme = url.strip_prefix("http://").or_else(|| url.strip_prefix("https://")).unwrap_or(url);
        if let Some(pos) = after_scheme.find('/') {
            let path = &after_scheme[pos..];
            let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
            return segments.last().unwrap_or(&"").to_string();
        }
    }
    let segments: Vec<&str> = url.split('/').filter(|s| !s.is_empty()).collect();
    segments.last().unwrap_or(&"").to_string()
}

fn extract_resource_name(path: &str) -> String {
    let segment = extract_path_segment(path);
    // Remove :id parameter
    if segment == ":id" || segment.is_empty() {
        let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        if segments.len() >= 2 {
            return segments[segments.len() - 2].to_string();
        }
        return "item".to_string();
    }
    segment
}

fn generate_single_object(resource: &str) -> String {
    let name_field = format!("{}Name", capitalize_first(resource.trim_end_matches('s')));
    format!(
        r#"{{"id":"{{$randomUuid}}","{}":"{{$randomFullName}}","email":"{{$randomEmail}}","createdAt":"{{$timestamp}}","status":"active"}}"#,
        name_field
    )
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
