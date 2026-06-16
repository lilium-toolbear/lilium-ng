use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=LILIUM_BUILD_GIT_HASH");
    register_git_rerun_paths();

    let commit = env::var("LILIUM_BUILD_GIT_HASH")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(current_git_commit_hash);

    if let Some(commit) = commit {
        println!("cargo:rustc-env=LILIUM_GIT_COMMIT_HASH={}", commit.trim());
    }
}

fn current_git_commit_hash() -> Option<String> {
    git_output(["rev-parse", "HEAD"])
}

fn register_git_rerun_paths() {
    if let Some(git_dir) = git_path(["rev-parse", "--git-dir"]) {
        println!("cargo:rerun-if-changed={}", git_dir.join("HEAD").display());
    }

    if let Some(common_dir) = git_path(["rev-parse", "--git-common-dir"]) {
        println!(
            "cargo:rerun-if-changed={}",
            common_dir.join("HEAD").display()
        );

        if let Some(head_ref) = git_output(["symbolic-ref", "-q", "HEAD"]) {
            println!(
                "cargo:rerun-if-changed={}",
                common_dir.join(head_ref.trim()).display()
            );
        }
    }
}

fn git_path<const N: usize>(args: [&str; N]) -> Option<PathBuf> {
    git_output(args).map(|path| {
        let path = PathBuf::from(path.trim());
        if path.is_absolute() {
            path
        } else {
            manifest_dir().join(path)
        }
    })
}

fn git_output<const N: usize>(args: [&str; N]) -> Option<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(manifest_dir())
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    String::from_utf8(output.stdout)
        .ok()
        .filter(|value| !value.trim().is_empty())
}

fn manifest_dir() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}
