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

fn get_current_branch() -> String {
    let output = run_command(Command::new("git").args(["branch", "--show-current"]));
    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if branch.is_empty() {
        eprintln!("Error: HEAD is detached (not on any branch)");
        exit(1);
    }

    branch
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
