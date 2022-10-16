# filewatch
A cli tool for performing actions on sets of files.

## Installation
`cargo install filewatch`bash

## Usage
`filewatch [OPTIONS] <FILE>`

## File Format

The rule file is a yaml file with the following format:

```
rename_and_move_test_files:
  interval:
  actions:
    - action: copy
      match_regex: (.*)/testfile_(.*)
      watch_dir: ./fixtures/test_data
      destination_dir: ./fixtures/test_data/backup
    - action: rename
      match_regex: test_[0-9]/testfile_(.*)
      rename_pattern: /renamed_$1
      watch_dir: ./fixtures/test_data
    - action: move
      match_regex: .*/renamed_(.*)
      watch_dir: ./fixtures/test_data
      destination_dir: ./fixtures/test_data/moved
    - action: delete
      match_regex: testfile_(.*)
      watch_dir: ./fixtures/test_data/backup
```yaml

## Options

`-h, --help` Prints help information

`-V, --version` Prints version information

`-v, --verbose` Sets the level of verbosity

`--dry-run` Runs the program without performing any actions

## License

filewatch is distributed under the terms of the MIT license.
