use std::process::Command;
use std::process::exit;

fn main() {
    run_command(Command::new("git").arg("add").arg("."));

    run_command(Command::new("git").arg("commit").arg("-m").arg("TODO"));

    println!("Committed with message: TODO");
}

fn run_command(cmd: &mut Command) {
    let status = cmd.status().expect("Failed to execute command");

    if !status.success() {
        eprintln!("Command failed with status: {}", status);
        exit(1);
    }
}
