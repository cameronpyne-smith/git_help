use dotenv::dotenv;
use std::env;
use std::process::exit;

use super::CommitMessageGenerator;

pub struct OpenAIProvider;

impl CommitMessageGenerator for OpenAIProvider {
    fn generate_commit_message(&self, diff_context: &str) -> String {
        dotenv().ok();

        let api_key = env::var("OPEN_AI_API_KEY").unwrap_or_else(|_| {
            eprintln!("Error: OPENAI_API_KEY environment variable not set.");
            eprintln!("Get a key at https://platform.openai.com/api-keys");
            exit(1);
        });

        let model = env::var("OPEN_AI_MODEL").unwrap_or_else(|_| "gpt-4.1-mini".to_string());

        let client = reqwest::blocking::Client::new();
        let url = "https://api.openai.com/v1/chat/completions";

        let body = serde_json::json!({
            "model": model,
            "messages": [
                {
                    "role": "system",
                    "content": "You are a commit message generator. Generate a concise conventional commit message. Return ONLY the commit message, no explanation."
                },
                {
                    "role": "user",
                    "content": diff_context
                }
            ],
            "temperature": 0.3
        });

        let resp = client
            .post(url)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&body)
            .send()
            .unwrap_or_else(|e| {
                eprintln!("API request failed: {}", e);
                exit(1);
            });

        let json: serde_json::Value = resp.json().unwrap();

        println!("open: {}", &json);

        json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("chore: update files")
            .trim()
            .to_string()
    }
}
