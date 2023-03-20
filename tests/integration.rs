use ::std::io::Write;
use assert_cmd::Command;

#[test]
fn it_runs() {
    // create a temporary file with some lines
    let mut file = tempfile::NamedTempFile::new().unwrap();
    let tmp_filename = file.path().to_str().unwrap().to_owned();
    file.write_all(b"line2\nline1\n").unwrap();
    file.flush().unwrap();

    // run with the temporary file as argument
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    let assert = cmd
        .arg(&tmp_filename)
        .arg("line4")
        .arg("line4")
        .arg("line3")
        .arg("line9")
        .arg("")
        .arg("line1")
        .assert();

    let expected_stdout = format!(
        "{} sorted and de-duplicated; delta: +3 lines\n",
        tmp_filename
    );
    let expected_stderr = "Warning: empty string passed as addition, skipping.\n".to_owned();
    assert
        .success()
        .stderr(expected_stderr)
        .stdout(expected_stdout);

    // cleanup the temporary file
    drop(file);
}
