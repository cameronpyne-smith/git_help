use dotenv::dotenv;

pub mod google;
pub mod openai;

pub trait CommitMessageGenerator {
    fn generate_commit_message(&self, diff_context: &str) -> String;
}

pub fn get_provider() -> Box<dyn CommitMessageGenerator> {
    dotenv().ok();
    let provider = std::env::var("AI_PROVIDER").unwrap_or_else(|_| "openai".to_string());

    match provider.as_str() {
        "google" => Box::new(google::GoogleProvider),
        "openai" => Box::new(openai::OpenAIProvider),
        _ => {
            eprintln!(
                "Unknown AI provider: '{}'. Supported: google, openai",
                provider
            );
            std::process::exit(1);
        }
    }
}
