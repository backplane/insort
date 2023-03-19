# insort


## Usage

The program prints the following usage text when invoked with the `-h` or `--help` arguments:

```
Utility which sorts the given file in-place and optionally inserts the given additions into the file

Usage: insort <filename> [additions]...

Arguments:
  <filename>      The file to sort and optionally insert additions into
  [additions]...  Optional string(s) to insert into the file (strings already in the file, will not be inserted)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### Example

Here's an example of adding `.env` and `**/.DS_Store` as lines in `.gitignore` and ensuring that `.gitignore` is sorted.

```sh
$ insort .gitignore '**/.DS_Store' .env
.gitignore sorted and 2 additions inserted.
```
