mod providers;

use std::env;
use std::process::exit;
use std::process::{Command, Output};

use crate::providers::get_provider;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        exit(1);
    }

    match args[1].as_str() {
        "commit" => commit(&args[2..]),
        "commit-ai" => commit_ai_message(),
        "pr" => pull_request(),
        "pr-ai" => pull_request_ai(),
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            print_usage();
            exit(1);
        }
    }
}

fn print_usage() {
    eprintln!("Usage: git_help <command> [args]");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  commit <message>   Stage all changes and commit with the given message");
    eprintln!("  pr [title]      Create a pull request for the current branch and open it");
}

//fn git_pull() {
//    run_command(Command::new("git").arg("pull"));
//}

fn git_add_all() {
    run_command(Command::new("git").args(["add", "."]));
}

fn git_diff() -> String {
    let output = run_command(Command::new("git").arg("diff"));
    String::from_utf8_lossy(&output.stdout).to_string()
}

//fn git_diff_cached() {
//    run_command(Command::new("git").args(["diff", "--cached"]));
//}

fn git_commit(message: &String) {
    run_command(Command::new("git").args(["commit", "-am", &message]));
    println!("Committed with message: {}", message);
}

fn commit(args: &[String]) {
    if args.is_empty() {
        eprintln!("Error: commit message is required");
        eprintln!("Usage: git_help commit <message>");
        exit(1);
    }

    git_add_all();

    let message = args.join(" ");
    git_commit(&message);
}

fn commit_ai_message() {
    let diff = git_diff();
    let provider = get_provider();
    let message = provider.generate_commit_message(&diff);
    git_add_all();
    git_commit(&message);
}

fn pull_request() {
    // TODO: Doesn't matter if git pull fails when no remote
    //git_pull();
    let branch = get_current_branch();

    if branch == "main" || branch == "master" {
        eprintln!(
            "Error: you are on the '{}' branch. Switch to a feature branch first.",
            branch
        );
        exit(1);
    }

    println!("Pushing branch '{}' to remote...", branch);
    run_command(
        Command::new("git")
            .arg("push")
            .arg("-u")
            .arg("origin")
            .arg(&branch),
    );

    let repo_url = get_pr_url(&branch);

    println!("Opening pull request page for branch '{}'...", branch);
    open_url(&repo_url);
}

fn pull_request_ai() {
    dotenv::dotenv().ok();

    let branch = get_current_branch();

    if branch == "main" || branch == "master" {
        eprintln!(
            "Error: you are on the '{}' branch. Switch to a feature branch first.",
            branch
        );
        exit(1);
    }

    commit_ai_message();
    run_command(
        Command::new("git")
            .arg("push")
            .arg("-u")
            .arg("origin")
            .arg(&branch),
    );

    let github_token = env::var("GITHUB_TOKEN").unwrap_or_else(|_| {
        eprintln!("Error: GITHUB_TOKEN environment variable not set.");
        eprintln!("Create a PAT at https://github.com/settings/tokens");
        exit(1);
    });

    let repo = get_github_repo();

    let title = "Title";
    let body = "Body";

    let client = reqwest::blocking::Client::new();
    let url = format!("https://api.github.com/repos/{}/pulls", repo);

    let payload = serde_json::json!({
        "title": title,
        "body": body,
        "head": branch,
        "base": "main" // TODO: might not be main? Git command or setting
    });

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", &github_token))
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "git_help")
        .json(&payload)
        .send()
        .unwrap_or_else(|e| {
            eprintln!("GitHub API request failed: {}", e);
            exit(1);
        });

    let status = resp.status();
    let body = resp.text().unwrap_or_default();

    if !status.is_success() {
        eprintln!("GitHub API error ({}): {}", status, body);
        exit(1);
    }

    let json: serde_json::Value = serde_json::from_str(&body).unwrap_or_else(|e| {
        eprintln!("Failed to parse response: {}\nBody: {}", e, body);
        exit(1);
    });

    if let Some(pr_url) = json["html_url"].as_str() {
        println!("Pull request created: {}", pr_url);
        open_url(pr_url);
    } else {
        eprintln!(
            "Failed to create PR: {}",
            serde_json::to_string_pretty(&json).unwrap()
        );
        exit(1);
    }
}

fn get_github_repo() -> String {
    let output = run_command(Command::new("git").args(["remote", "get-url", "origin"]));
    let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();

    raw.replace("git@github.com:", "")
        .replace("https://github.com/", "")
        .trim_end_matches(".git")
        .to_string()
}

fn get_pr_url(branch: &String) -> String {
    let output = run_command(Command::new("git").args(["remote", "get-url", "origin"]));
    let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();

    let base_url = raw
        .replace("git@github.com:", "https://github.com/")
        .trim_end_matches(".git")
        .to_string();

    let url = format!("{}/pull/new/{}", base_url, branch);

    url
}

fn get_current_branch() -> String {
    let output = run_command(Command::new("git").args(["branch", "--show-current"]));
    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if branch.is_empty() {
        eprintln!("Error: HEAD is detached (not on any branch)");
        exit(1);
    }

    branch
}

fn open_url(url: &str) {
    #[cfg(target_os = "windows")]
    let result = Command::new("cmd").args(["/C", "start", url]).status();

    #[cfg(target_os = "macos")]
    let result = Command::new("open").arg(url).status();

    #[cfg(target_os = "linux")]
    let result = Command::new("xdg-open").arg(url).status();

    match result {
        Ok(status) if status.success() => println!("Opened {} in browser", url),
        Ok(_) => eprintln!("Failed to open URL in browser"),
        Err(e) => eprintln!("Failed to open URL: {}", e),
    }
}

fn run_command(cmd: &mut Command) -> Output {
    let output = cmd.output().expect("Failed to execute command");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Command failed with status: {}\n{}", output.status, stderr);
        exit(1);
    }

    output
}
