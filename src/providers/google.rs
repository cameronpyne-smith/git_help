use dotenv::dotenv;
use std::env;
use std::process::exit;

use super::CommitMessageGenerator;

pub struct GoogleProvider;

impl CommitMessageGenerator for GoogleProvider {
    fn generate_commit_message(&self, diff_context: &str) -> String {
        let prompt = format!(
            "Generate a concise conventional commit message for these changes.\n\
             Return ONLY the commit message, no explanation.\n\n{}",
            diff_context
        );
        call_api(&prompt)
    }

    fn generate_pr_title(&self, commit_log: &str) -> String {
        let prompt = format!(
            "Generate a concise pull request title summarizing these commits.\n\
             Return ONLY the title, no explanation. Keep it under 72 characters.\n\n{}",
            commit_log
        );
        call_api(&prompt)
    }

    fn generate_pr_body(&self, commit_log: &str) -> String {
        let prompt = format!(
            "Generate a pull request description in markdown for these commits.\n\
             Include a brief summary, then a list of changes.\n\
             Return ONLY the markdown body, no extra explanation.\n\n{}",
            commit_log
        );
        call_api(&prompt)
    }
}

fn call_api(prompt: &str) -> String {
    dotenv().ok();

    let api_key = env::var("GOOGLE_API_KEY").unwrap_or_else(|_| {
        eprintln!("Error: GOOGLE_API_KEY environment variable not set.");
        eprintln!("Get a free key at https://aistudio.google.com/apikey");
        exit(1);
    });

    let model = env::var("GOOGLE_MODEL").unwrap_or_else(|_| "gemini-3-flash-preview".to_string());

    let client = reqwest::blocking::Client::new();
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
        model
    );

    let body = serde_json::json!({
        "contents": [{
            "parts": [{
                "text": prompt
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
