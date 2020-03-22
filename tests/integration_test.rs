use std::process::Command;

use assert_cmd::prelude::*;

#[test]
fn it_fails_when_file_does_not_exist() -> Result<(), Box<dyn std::error::Error>> {
    let mut command = Command::cargo_bin("dj-library-gain-calculator")?;

    command.arg("tests/vectors/not_found.nml");

    command
        .assert()
        .failure()
        .stdout("No such file or directory (os error 2)\naborting…\n");

    Ok(())
}
