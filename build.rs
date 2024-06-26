use std::env;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set a build-time `GIT_HEAD` env var which includes the commit id;
    // such that we can tell which code is running.
    let hash = Command::new("git")
        .arg("rev-parse")
        .arg("--short")
        .arg("HEAD")
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .unwrap_or(env::var("GIT_HEAD").unwrap_or("unknown".into()));

    let tags = Command::new("git")
        .arg("tag")
        .arg("--points-at")
        .arg("HEAD")
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .unwrap_or_default();
    let tags = tags
        .split_terminator('\n')
        .filter_map(|s| s.strip_prefix('v'))
        .collect::<Vec<_>>();

    if tags.len() > 1 {
        return Err("More than one version tag found for commit {hash}: {tags:?}".into());
    }
    // Used for `RADICLE_VERSION` env.
    let version = if let Some(version) = tags.first() {
        // There's a tag pointing at this `HEAD`.
        // Eg. "1.0.43".
        Some((*version).to_owned())
    } else {
        // If `HEAD` doesn't have a tag pointing to it, this is a development version,
        // so find the closest tag starting with `v` and append `-dev` to the version.
        // Eg. "1.0.43-dev".
        Command::new("git")
            .arg("describe")
            .arg("--match=v*") // Match tags starting with `v`
            .arg("--candidates=1") // Only one result
            .arg("--abbrev=0") // Don't add the commit short-hash to the tag name
            .arg("HEAD")
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    String::from_utf8(output.stdout).ok()
                } else {
                    None
                }
            })
            .map(|last| format!("{}-dev", last.trim()))
    }
    // If there are no tags found, we'll just call this a pre-release.
    .unwrap_or(String::from("pre-release"));

    // Set a build-time `GIT_COMMIT_TIME` env var which includes the commit time.
    let commit_time = Command::new("git")
        .arg("show")
        .arg("--format=%ct")
        .arg("HEAD")
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .unwrap_or(0.to_string());

    println!("cargo::rustc-env=RADICLE_VERSION={version}");
    println!("cargo::rustc-env=GIT_COMMIT_TIME={commit_time}");
    println!("cargo::rustc-env=GIT_HEAD={hash}");

    Ok(())
}
