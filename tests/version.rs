use std::process::Command;

#[test]
fn version_flag_prints_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_saints-mile"))
        .arg("--version")
        .output()
        .expect("failed to run binary");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(env!("CARGO_PKG_VERSION")));
    assert!(output.status.success());
}

#[test]
fn help_flag_prints_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_saints-mile"))
        .arg("--help")
        .output()
        .expect("failed to run binary");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("USAGE:"));
    assert!(output.status.success());
}

#[test]
fn cargo_version_is_valid_semver() {
    let version = env!("CARGO_PKG_VERSION");
    let parts: Vec<&str> = version.split('.').collect();
    assert_eq!(parts.len(), 3, "version must be semver: {}", version);
    for part in &parts {
        part.parse::<u32>().unwrap_or_else(|_| panic!("non-numeric semver part: {}", part));
    }
}
