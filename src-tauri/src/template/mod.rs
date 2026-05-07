use once_cell::sync::Lazy;
use rand::Rng;
use regex::Regex;
use std::collections::HashMap;

static TEMPLATE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\{\{(.+?)\}\}").expect("Invalid regex"));

pub fn render_template(template: &str, context: &HashMap<String, String>) -> String {
    TEMPLATE_RE
        .replace_all(template, |caps: &regex::Captures| {
            let key = caps.get(1).map(|m| m.as_str().trim()).unwrap_or("");
            match key {
                "$randomUuid" | "$guid" => uuid::Uuid::new_v4().to_string(),
                "$timestamp" => std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs().to_string())
                    .unwrap_or_else(|_| "0".to_string()),
                "$randomInt" => rand::thread_rng().gen_range(0..1000).to_string(),
                "$randomBoolean" => {
                    if rand::thread_rng().gen_bool(0.5) { "true" } else { "false" }.to_string()
                }
                "$randomFullName" => generate_random_name(),
                "$randomEmail" => generate_random_email(),
                _ => context.get(key).cloned().unwrap_or_else(|| caps[0].to_string()),
            }
        })
        .to_string()
}

fn generate_random_name() -> String {
    let first_names = [
        "James", "Mary", "John", "Patricia", "Robert", "Jennifer", "Michael", "Linda",
        "William", "Elizabeth", "David", "Barbara", "Richard", "Susan", "Joseph", "Jessica",
        "Thomas", "Sarah", "Charles", "Karen", "Christopher", "Nancy", "Daniel", "Lisa",
        "Matthew", "Betty", "Anthony", "Margaret", "Mark", "Sandra",
    ];
    let last_names = [
        "Smith", "Johnson", "Williams", "Brown", "Jones", "Garcia", "Miller", "Davis",
        "Rodriguez", "Martinez", "Hernandez", "Lopez", "Gonzalez", "Wilson", "Anderson",
        "Thomas", "Taylor", "Moore", "Jackson", "Martin", "Lee", "Perez", "Thompson", "White",
    ];

    let mut rng = rand::thread_rng();
    let first = first_names[rng.gen_range(0..first_names.len())];
    let last = last_names[rng.gen_range(0..last_names.len())];
    format!("{} {}", first, last)
}

fn generate_random_email() -> String {
    let domains = ["example.com", "test.com", "demo.org", "sample.net"];
    let mut rng = rand::thread_rng();
    let name = format!(
        "user{}",
        rng.gen_range(100..9999)
    );
    let domain = domains[rng.gen_range(0..domains.len())];
    format!("{}@{}", name, domain)
}
