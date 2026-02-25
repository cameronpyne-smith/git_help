use std::env;
use std::process::Command;
use std::process::exit;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        exit(1);
    }

    match args[1].as_str() {
        "commit" => commit(&args[2..]),
        "pr" => pull_request(&args[2..]),
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

/// Stage all changes and commit with the provided message.
fn commit(args: &[String]) {
    if args.is_empty() {
        eprintln!("Error: commit message is required");
        eprintln!("Usage: git_help commit <message>");
        exit(1);
    }

    let message = args.join(" ");

    run_command(Command::new("git").arg("add").arg("."));
    run_command(Command::new("git").arg("commit").arg("-m").arg(&message));

    println!("Committed with message: {}", message);
}

/// Create a GitHub pull request for the current branch and open the URL.
fn pull_request(args: &[String]) {
    // Push the current branch first to make sure remote is up to date
    let branch = get_current_branch();
    println!("Pushing branch '{}' to remote...", branch);
    run_command(
        Command::new("git")
            .arg("push")
            .arg("--set-upstream")
            .arg("origin")
            .arg(&branch),
    );

    // Build the gh pr create command
    let title = if args.is_empty() {
        branch.clone()
    } else {
        args.join(" ")
    };

    println!("Creating pull request: {}", title);

    let output = Command::new("gh")
        .args(["pr", "create", "--title", &title, "--body", "", "--fill"])
        .output()
        .expect("Failed to execute 'gh' CLI. Is GitHub CLI installed?");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // If a PR already exists, try to get its URL instead
        if stderr.contains("already exists") {
            println!("A pull request already exists for this branch.");
            open_existing_pr();
            return;
        }
        eprintln!("Failed to create pull request: {}", stderr);
        exit(1);
    }

    let pr_url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    println!("Pull request created: {}", pr_url);

    if !pr_url.is_empty() {
        open_url(&pr_url);
    }
}

/// Get the current git branch name.
fn get_current_branch() -> String {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .expect("Failed to get current branch");

    if !output.status.success() {
        eprintln!("Failed to determine current branch");
        exit(1);
    }

    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

/// Open the existing PR for the current branch in the browser.
fn open_existing_pr() {
    let output = Command::new("gh")
        .args(["pr", "view", "--web"])
        .status()
        .expect("Failed to open existing pull request");

    if !output.success() {
        eprintln!("Failed to open existing pull request in browser");
        exit(1);
    }
}

/// Open a URL in the default browser.
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

fn run_command(cmd: &mut Command) {
    let status = cmd.status().expect("Failed to execute command");

    if !status.success() {
        eprintln!("Command failed with status: {}", status);
        exit(1);
    }
}
