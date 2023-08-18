use clap::Parser;
use std::fs::File;
use std::io::{stdin, stdout, BufRead, BufReader, Read, Write};

//version constant from Cargo.toml
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(name = "insort", version = VERSION, author = "Backplane BV", about = "Utility which sorts the given file in-place and optionally inserts the given additions into the file")]
struct Cli {
    /// Create the output file if it doesn't already exist
    #[clap(short = 'c', long = "create", conflicts_with = "no_create")]
    create: bool,

    /// Don't create the output file if it doesn't already exist
    #[clap(short = 'n', long = "no-create", conflicts_with = "create")]
    no_create: bool,

    /// The file to sort and optionally insert additions into
    #[clap(name = "filename", required = true)]
    filename: String,

    /// Optional string(s) to insert into the file (strings already in the file will not be inserted)
    #[clap(name = "additions", required = false)]
    additions: Vec<String>,
}
fn main() {
    // parse the command line arguments
    let cli: Cli = Cli::parse();
    let filename = &cli.filename;
    let additions = &cli.additions;
    let create: Option<bool> = match (cli.create, cli.no_create) {
        (true, false) => Some(true),
        (false, true) => Some(false),
        _ => None,
    };

    // call insert_and_sort handling possible errors
    if let Err(e) = insert_and_sort(filename, additions, create) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    // return an exit code
    std::process::exit(0);
}

// read contents of the file with the given name into a String
fn read_file(filename: &str) -> std::io::Result<String> {
    // Open the file
    let mut file = File::open(filename)?;

    // Read the file contents into a String
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    Ok(contents)
}

/// prompt_user: interactively asks the user if it's ok to create the given file on the given reader and writer
fn prompt_user<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
    filename: &str,
) -> Result<bool, std::io::Error> {
    let mut reader = BufReader::new(reader);
    let mut input = String::new();
    write!(writer, "{} does not exist. create it? (y/n) >", filename)?;
    writer.flush()?;
    reader.read_line(&mut input)?;
    Ok(input.trim() == "y")
}

/// prompt_user_std: interactively asks the user if it's ok to create the given file on STDOUT and STDIN
fn prompt_user_std(filename: &str) -> Result<bool, std::io::Error> {
    let stdin = stdin();
    let stdout = stdout();
    prompt_user(&mut stdin.lock(), &mut stdout.lock(), filename)
}

/// insert_and_sort: inserts the arguments into the file with the given name and sorts the file (in-place), returns an error if the file cannot be opened or written to
fn insert_and_sort(
    filename: &str,
    additions: &Vec<String>,
    allow_create: Option<bool>,
) -> Result<(), std::io::Error> {
    let contents = match read_file(filename) {
        Ok(contents) => contents,
        Err(err) => match err.kind() {
            std::io::ErrorKind::NotFound => match allow_create {
                Some(true) => String::new(),
                Some(false) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "File not found and creation is not allowed",
                    ))
                }
                None => {
                    let decision = prompt_user_std(filename)?;
                    if decision {
                        String::new()
                    } else {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "File not found and user chose not to create it",
                        ));
                    }
                }
            },
            _ => return Err(err),
        },
    };

    // load the lines into a vector, removing empty lines
    let mut lines: Vec<String> = contents
        .lines()
        .map(|line| line.to_string())
        .filter(|line| !line.is_empty())
        .collect();
    let original_lines = lines.clone();

    // insert any given additions into the vector
    for addition in additions {
        if addition.is_empty() {
            eprintln!("Warning: empty string passed as addition, skipping.");
            continue;
        }
        lines.push(addition.to_string());
    }

    // sort and deduplicate the vector
    lines.sort();
    lines.dedup();

    // determine if the contents of the vector have changed (handling the case where the file was originally empty)
    let mut changed = false;
    if lines.len() != original_lines.len() {
        changed = true;
    } else {
        for i in 0..lines.len() {
            if lines[i] != original_lines[i] {
                changed = true;
                break;
            }
        }
    }
    if !changed {
        println!("{} left unchanged.", filename);
        return Ok(());
    }

    let updated_lines_len = lines.len();

    // write the vector back to the file (in-place)
    let mut file = File::create(filename)?;
    file.write_all(lines.join("\n").as_bytes())?;
    file.write_all(b"\n")?;
    file.flush()?;

    // report the number of lines added or removed in the file with the filename
    let lines_delta = updated_lines_len as i32 - original_lines.len() as i32;
    println!(
        "{} sorted and de-duplicated; delta: {}{} {}",
        filename,
        if lines_delta < 0 { "-" } else { "+" },
        lines_delta,
        if lines_delta == 1 { "line" } else { "lines" },
    );

    // return Ok
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Cursor;
    use std::io::Write;

    #[test]
    fn test_prompt_user_yes() {
        let input = Cursor::new("y\n");
        let mut output = Vec::new();

        let decision = prompt_user(&mut input.clone(), &mut output, "test.txt").unwrap();

        assert_eq!(decision, true);
        assert_eq!(
            String::from_utf8(output).unwrap(),
            "test.txt does not exist. create it? (y/n) >"
        );
    }

    #[test]
    fn test_prompt_user_no() {
        let input = Cursor::new("n\n");
        let mut output = Vec::new();

        let decision = prompt_user(&mut input.clone(), &mut output, "test.txt").unwrap();

        assert_eq!(decision, false);
        assert_eq!(
            String::from_utf8(output).unwrap(),
            "test.txt does not exist. create it? (y/n) >"
        );
    }

    #[test]
    fn test_read_file() {
        let data = "Hello, world!\n";

        // create a temporary file
        let mut file = tempfile::NamedTempFile::new().unwrap();
        let tmp_filename = file.path().to_str().unwrap().to_owned();

        // write some lines to the file
        file.write_all(data.as_bytes()).unwrap();
        // flush the file
        file.flush().unwrap();

        // Test that read_file returns the correct contents
        let contents = read_file(&tmp_filename).unwrap();
        assert_eq!(contents, data);
    }

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
        insert_and_sort(&tmp_filename, &vec!["line3".to_string()], Some(false)).unwrap();

        // reopen the file
        let mut file = File::open(&tmp_filename).unwrap();

        // read the file back into a string
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        // check the contents
        assert_eq!(contents, "line1\nline2\nline3\n".to_string());

        // cleanup the temporary file
        drop(file);

        // ----------------------------------------------
        // call insert_and_sort again, this time with no additions on an empty file

        // create a new empty temp file
        let file = tempfile::NamedTempFile::new().unwrap();
        let tmp_filename = file.path().to_str().unwrap().to_owned();

        // call insert_and_sort again, this time with no additions on an empty file
        insert_and_sort(&tmp_filename, &vec![], Some(false)).unwrap();

        // reopen the file
        let mut file = File::open(&tmp_filename).unwrap();

        // read the file back into a string
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        // check the contents
        assert_eq!(contents, "".to_string());

        // cleanup the temporary file
        drop(file);

        // ----------------------------------------------
        // call insert_and_sort again, this time with no additions on a file with one line

        // create a new empty temp file
        let mut file = tempfile::NamedTempFile::new().unwrap();
        let tmp_filename = file.path().to_str().unwrap().to_owned();

        // write a line to the file
        file.write_all(b"line1\n").unwrap();
        // flush the file
        file.flush().unwrap();

        // call insert_and_sort again, this time with no additions on a file with one line
        insert_and_sort(&tmp_filename, &vec![], Some(false)).unwrap();

        // reopen the file
        let mut file = File::open(&tmp_filename).unwrap();

        // read the file back into a string
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        // check the contents
        assert_eq!(contents, "line1\n".to_string());

        // cleanup the temporary file
        drop(file);

        // ----------------------------------------------
        // call insert_and_sort again, this time with one addition on an empty file

        // create a new empty temp file
        let file = tempfile::NamedTempFile::new().unwrap();
        let tmp_filename = file.path().to_str().unwrap().to_owned();

        // call insert_and_sort again, this time with one addition on an empty file
        insert_and_sort(&tmp_filename, &vec!["line1".to_string()], Some(false)).unwrap();

        // reopen the file
        let mut file = File::open(&tmp_filename).unwrap();

        // read the file back into a string
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        // check the contents
        assert_eq!(contents, "line1\n".to_string());

        // cleanup the temporary file
        drop(file);
    }
}
