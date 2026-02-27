use dotenv::dotenv;
use std::env;
use std::process::exit;

pub fn generate_commit_message(diff_context: &str) -> String {
    dotenv().ok();

    let api_key = env::var("GEMINI_API_KEY").unwrap_or_else(|_| {
        eprintln!("Error: GEMINI_API_KEY environment variable not set.");
        eprintln!("Get a free key at https://aistudio.google.com/apikey");
        exit(1);
    });

    let model = env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-3-flash-preview".to_string());

    let client = reqwest::blocking::Client::new();
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
        model
    );

    let body = serde_json::json!({
        "contents": [{
            "parts": [{
                "text": format!(
                    "Generate a concise conventional commit message for these changes.\n\
                     Return ONLY the commit message, no explanation.\n\n{}",
                    diff_context
                )
            }]
        }]
    });

    let resp = client
        .post(&url)
        .header("x-goog-api-key", &api_key)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .unwrap_or_else(|e| {
            eprintln!("API request failed: {}", e);
            exit(1);
        });

    let json: serde_json::Value = resp.json().unwrap();

    json["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .unwrap_or("chore: update files")
        .trim()
        .to_string()
}
