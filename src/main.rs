use clap::Parser;
use std::fs::File;
use std::io::{Read, Write};

//version constant from Cargo.toml
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(name = "insort", version = VERSION, author = "Backplane BV", about = "Utility which sorts the given file in-place and optionally inserts the given additions into the file")]
struct Cli {
    /// The file to sort and optionally insert additions into
    #[clap(name = "filename", required = true)]
    filename: String,

    /// Optional string(s) to insert into the file (strings already in the file, will not be inserted)
    #[clap(name = "additions", required = false)]
    additions: Vec<String>,
}
fn main() {
    // parse the command line arguments
    let cli: Cli = Cli::parse();
    let filename = &cli.filename;
    let additions = &cli.additions;

    // call insert_and_sort handling possible errors
    if let Err(e) = insert_and_sort(filename, additions) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    // return an exit code
    std::process::exit(0);
}

// insert_and_sort: inserts the arguments into the file with the given name and sorts the file (in-place), returns an error if the file cannot be opened or written to
fn insert_and_sort(filename: &str, additions: &Vec<String>) -> Result<(), std::io::Error> {
    // open the file
    let mut file = File::open(filename)?;

    // read the lines into a vector
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let mut lines: Vec<&str> = contents.split('\n').collect();

    // (optionally) insert the arguments into the vector
    for addition in additions {
        // skip additions that are already in the file
        if lines.contains(&addition.as_str()) {
            continue;
        }
        lines.push(addition);
    }

    // sort the vector
    lines.sort();

    // determine if the vector has changed
    let mut changed = false;
    for (i, line) in lines.iter().enumerate() {
        if line != &contents.split('\n').collect::<Vec<&str>>()[i] {
            changed = true;
            break;
        }
    }

    // if the vector has not changed, return Ok
    if !changed {
        eprintln!("{} left unchanged.", filename);
        return Ok(());
    }

    // write the vector back to the file (in-place) with the same permissions
    let mut file = File::create(filename)?;
    for line in lines {
        // skip blank lines
        if line.is_empty() {
            continue;
        }
        file.write_all(line.as_bytes())?;
        // write a newline to the buffer
        file.write_all(b"\n")?;
    }

    // close the file
    file.flush()?;

    // report the changes
    eprintln!(
        "{} sorted and {} additions inserted.",
        filename,
        additions.len()
    );

    // return Ok
    Ok(())
}

// function to test the insert_and_sort function
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_sort() {
        // create a temporary file
        let mut file = tempfile::NamedTempFile::new().unwrap();
        let tmp_filename = file.path().to_str().unwrap().to_owned();

        // write some lines to the file
        file.write_all(b"line2\nline1\n").unwrap();
        // flush the file
        file.flush().unwrap();

        // call insert_and_sort
        insert_and_sort(
            &tmp_filename,
            &vec![
                "line4".to_string(),
                "line4".to_string(),
                "line3".to_string(),
                "line9".to_string(),
            ],
        )
        .unwrap();

        // reopen the file
        let mut file = File::open(&tmp_filename).unwrap();

        // read the file back into a string
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        // check the contents
        assert_eq!(contents, "line1\nline2\nline3\nline4\nline9\n".to_string());
    }
}
