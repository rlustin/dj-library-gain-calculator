use std::process::Command;
use std::fs::read_to_string;

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

#[test]
fn it_parses_a_1_element_collection() -> Result<(), Box<dyn std::error::Error>> {
    let mut command = Command::cargo_bin("dj-library-gain-calculator")?;

    let input_path = "tests/vectors/1_element_collection.nml";
    command.arg(input_path);
    let mut string = read_to_string(input_path)?;
    // The serializer doesn't produce a \n at the end, remove it from the file before comparing
    string.truncate(string.len() - 1);

    command
        .assert()
        .success()
        .stdout(string);

    Ok(())
}
