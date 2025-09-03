use std::process::Command;

fn main() {
    let output = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .unwrap_or_else(|_| panic!("Failed to get git commit hash"));

    let git_hash = String::from_utf8_lossy(&output.stdout);

    let git_hash = git_hash.trim();
    let git_hash = if git_hash.is_empty() { "unknown" } else { git_hash };

    println!("cargo:rustc-env=GIT_HASH={}", git_hash);
}
