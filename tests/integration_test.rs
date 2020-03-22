use assert_cmd::prelude::*;
use std::fs::read_to_string;
use std::process::Command;
use tempdir::TempDir;

#[test]
fn it_fails_when_file_does_not_exist() -> Result<(), Box<dyn std::error::Error>> {
    let mut command = Command::cargo_bin("dj-library-gain-calculator")?;

    command.arg("--input").arg("tests/vectors/not_found.nml");

    command
        .assert()
        .failure()
        .stdout("No such file or directory (os error 2)\naborting…\n");

    Ok(())
}

#[test]
fn it_processes_a_1_element_collection_to_stdout() -> Result<(), Box<dyn std::error::Error>> {
    let mut command = Command::cargo_bin("dj-library-gain-calculator")?;

    let input_path = "tests/vectors/1_element_collection.nml";
    let mut input_content = read_to_string(input_path)?;
    // The serializer doesn't produce a \n at the end, remove it from the file before comparing
    input_content.truncate(input_content.len() - 1);

    command
        .arg("--input")
        .arg(input_path)
        .assert()
        .success()
        .stdout(input_content);

    Ok(())
}

#[test]
fn it_processes_a_1_element_collection_to_a_file() -> Result<(), Box<dyn std::error::Error>> {
    let mut command = Command::cargo_bin("dj-library-gain-calculator")?;

    let input_path = "tests/vectors/1_element_collection.nml";
    let output_dir = TempDir::new("tests").unwrap();
    let output_path = output_dir.path().join("output.nml");

    command
        .arg("--input")
        .arg(input_path)
        .arg("--output")
        .arg(&output_path)
        .assert()
        .success();

    let mut input_content = read_to_string(input_path)?;
    // The serializer doesn't produce a \n at the end, remove it from the file before comparing
    input_content.truncate(input_content.len() - 1);

    let output_content = read_to_string(output_path)?;

    assert_eq!(input_content, output_content);

    Ok(())
}
