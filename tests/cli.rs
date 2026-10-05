//! End-to-end checks of the binary on retained certificates.
//!
//! `small-certificate` is a complete 83-node certificate; each `mutated-*` copy changes one
//! field of it, and the verifier must reject that copy at the corresponding check.

use std::path::PathBuf;
use std::process::Command;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn run(name: &str) -> (i32, serde_json::Value) {
    let output = Command::new(env!("CARGO_BIN_EXE_n17bb-verify"))
        .arg(fixture(name))
        .args(["--threads", "2"])
        .output()
        .expect("the verifier binary runs");
    let receipt = serde_json::from_slice(&output.stdout).expect("the receipt is JSON");
    (output.status.code().expect("an exit code"), receipt)
}

fn first_failure(receipt: &serde_json::Value) -> String {
    receipt["failures"][0]
        .as_str()
        .unwrap_or_default()
        .to_owned()
}

#[test]
fn complete_certificate_passes() {
    let (code, receipt) = run("small-certificate");
    assert_eq!(code, 0);
    assert_eq!(receipt["status"], "PASS");
    assert_eq!(receipt["nodes"], 83);
    assert_eq!(receipt["node_failures"], 0);
}

#[test]
fn corrupted_certificates_fail_at_their_check() {
    for (name, needle) in [
        ("mutated-multiplier", "negative multiplier"),
        ("mutated-bound", "C4: bound on column"),
        ("mutated-cut", "C2: cut"),
        ("mutated-split", "T2:"),
        ("mutated-drop-leaf", "T2:"),
        ("mutated-narrow-final", "B2: final box too small"),
    ] {
        let (code, receipt) = run(name);
        assert_eq!(code, 1, "{name} must fail");
        assert_eq!(receipt["status"], "FAIL", "{name}");
        let failure = first_failure(&receipt);
        assert!(
            failure.contains(needle),
            "{name}: unexpected failure {failure:?}"
        );
    }
}
