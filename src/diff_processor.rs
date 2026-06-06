use std::process::{Command, exit};

const MAX_LINES_PER_FILE: usize = 50;
const MAX_TOTAL_DIFF_LINES: usize = 500;

/// Pre-process a raw git diff to reduce token usage while preserving intent signals.
///
/// Strategy:
/// 1. File-level summary (--name-status + --stat)
/// 2. Per-file hunks, capped to MAX_LINES_PER_FILE each
/// 3. Whitespace-only and comment-only files collapsed to a note
/// 4. Total output capped to MAX_TOTAL_DIFF_LINES
pub fn preprocess_diff(diff_args: &[&str]) -> String {
    let name_status = git_name_status(diff_args);
    let stat = git_stat(diff_args);
    let raw_diff = git_raw_diff(diff_args);

    let file_diffs = parse_file_diffs(&raw_diff);

    let mut sections: Vec<String> = Vec::new();

    sections.push("## Files Changed".to_string());
    sections.push(name_status);
    sections.push(String::new());
    sections.push("## Change Summary".to_string());
    sections.push(stat);
    sections.push(String::new());
    sections.push("## Diff Details".to_string());

    let mut total_lines: usize = sections.iter().map(|s| s.lines().count()).sum();

    for file_diff in &file_diffs {
        if total_lines >= MAX_TOTAL_DIFF_LINES {
            sections.push(format!(
                "\n... {} more file(s) truncated (token budget reached)",
                file_diffs.len() - sections.len()
            ));
            break;
        }

        let processed = process_file_diff(file_diff);
        let line_count = processed.lines().count();

        total_lines += line_count;
        sections.push(processed);
    }

    sections.join("\n")
}

/// Classify whether a hunk contains only whitespace or comment changes.
fn is_noise_only(hunk_lines: &[&str]) -> bool {
    for line in hunk_lines {
        if line.starts_with("@@") || line.is_empty() {
            continue;
        }

        let is_change = line.starts_with('+') || line.starts_with('-');
        if !is_change {
            continue;
        }

        // Strip the +/- prefix and trim
        let content = line[1..].trim();

        // Skip empty lines
        if content.is_empty() {
            continue;
        }

        // Skip comment-only lines (common patterns across languages)
        if content.starts_with("//")
            || content.starts_with('#')
            || content.starts_with("/*")
            || content.starts_with("* ")
            || content.starts_with("*/")
            || content.starts_with("<!--")
            || content.starts_with("-->")
        {
            continue;
        }

        // If we get here, there's a meaningful change
        return false;
    }

    true
}

/// Process a single file's diff: collapse noise, cap lines, keep hunk headers.
fn process_file_diff(file_diff: &str) -> String {
    let lines: Vec<&str> = file_diff.lines().collect();

    if lines.is_empty() {
        return String::new();
    }

    // Extract file header (--- and +++ lines)
    let mut header_lines: Vec<&str> = Vec::new();
    let mut hunk_start = 0;

    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("@@") {
            hunk_start = i;
            break;
        }
        header_lines.push(line);
        hunk_start = i + 1;
    }

    let hunk_lines = &lines[hunk_start..];

    // Check if entire file diff is just noise
    if is_noise_only(hunk_lines) {
        let filename = extract_filename(&header_lines);
        return format!("### {} (whitespace/comment changes only)", filename);
    }

    // Split into hunks and process each
    let hunks = split_hunks(hunk_lines);
    let mut result_lines: Vec<String> = header_lines.iter().map(|l| l.to_string()).collect();
    let mut file_line_count = result_lines.len();

    for hunk in &hunks {
        if file_line_count >= MAX_LINES_PER_FILE {
            result_lines.push(format!(
                "  ... {} more hunk(s) truncated",
                hunks.len()
            ));
            break;
        }

        // Skip noise-only hunks
        if is_noise_only(hunk) {
            if let Some(header) = hunk.first() {
                if header.starts_with("@@") {
                    result_lines.push(format!("{} (whitespace/comment changes only)", header));
                    file_line_count += 1;
                }
            }
            continue;
        }

        // Keep the hunk header (contains function context)
        // Then include change lines, preferring signatures over bodies
        for line in hunk {
            if file_line_count >= MAX_LINES_PER_FILE {
                result_lines.push("  ... hunk truncated".to_string());
                break;
            }
            result_lines.push(line.to_string());
            file_line_count += 1;
        }
    }

    result_lines.join("\n")
}

/// Split hunk lines into individual hunks by @@ markers.
fn split_hunks<'a>(lines: &'a [&'a str]) -> Vec<Vec<&'a str>> {
    let mut hunks: Vec<Vec<&str>> = Vec::new();
    let mut current: Vec<&str> = Vec::new();

    for line in lines {
        if line.starts_with("@@") && !current.is_empty() {
            hunks.push(current);
            current = Vec::new();
        }
        current.push(line);
    }

    if !current.is_empty() {
        hunks.push(current);
    }

    hunks
}

/// Extract filename from diff header lines.
fn extract_filename(header: &[&str]) -> String {
    for line in header {
        if line.starts_with("+++ b/") {
            return line[6..].to_string();
        }
        if line.starts_with("diff --git") {
            let parts: Vec<&str> = line.split(" b/").collect();
            if parts.len() > 1 {
                return parts[1].to_string();
            }
        }
    }
    "unknown".to_string()
}

/// Parse a raw diff into per-file sections.
fn parse_file_diffs(raw_diff: &str) -> Vec<String> {
    let mut files: Vec<String> = Vec::new();
    let mut current: Vec<&str> = Vec::new();

    for line in raw_diff.lines() {
        if line.starts_with("diff --git") && !current.is_empty() {
            files.push(current.join("\n"));
            current = Vec::new();
        }
        current.push(line);
    }

    if !current.is_empty() {
        files.push(current.join("\n"));
    }

    files
}

fn git_name_status(diff_args: &[&str]) -> String {
    let mut args = vec!["diff", "--name-status"];
    args.extend_from_slice(diff_args);

    let output = Command::new("git")
        .args(&args)
        .output()
        .expect("Failed to execute git diff --name-status");

    if !output.status.success() {
        eprintln!("git diff --name-status failed");
        exit(1);
    }

    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn git_stat(diff_args: &[&str]) -> String {
    let mut args = vec!["diff", "--stat"];
    args.extend_from_slice(diff_args);

    let output = Command::new("git")
        .args(&args)
        .output()
        .expect("Failed to execute git diff --stat");

    if !output.status.success() {
        eprintln!("git diff --stat failed");
        exit(1);
    }

    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn git_raw_diff(diff_args: &[&str]) -> String {
    let mut args = vec!["diff", "-U3"];
    args.extend_from_slice(diff_args);

    let output = Command::new("git")
        .args(&args)
        .output()
        .expect("Failed to execute git diff");

    if !output.status.success() {
        eprintln!("git diff failed");
        exit(1);
    }

    String::from_utf8_lossy(&output.stdout).to_string()
}
