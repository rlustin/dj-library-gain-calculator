extern crate assert_cmd;

use assert_cmd::prelude::*;
use std::process::Command;

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

#[test]
fn it_parses_a_1_element_collection() -> Result<(), Box<dyn std::error::Error>> {
    let mut command = Command::cargo_bin("dj-library-gain-calculator")?;

    command.arg("tests/vectors/1_element_collection.nml");

    command
        .assert()
        .success()
        .stdout("Lock'N Load — Blow Ya Mind (Club Caviar Mix)\n");

    Ok(())
}
