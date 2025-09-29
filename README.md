## Project Documentation

- [Division of Labor](Doc/division-of-labor.pdf)

## File Structure
- `src/` - Source code directory
  - `main.rs` - Entry point of the program
  - `parser.rs` - Parses input job specifications
  - `job.rs` - Job data structure and related functions
  - `executor.rs` - Handles job execution logic
  - `builtins.rs` - Built-in command implementations
  - `job.rs.swo` - Swap file (temporary editor file)
  - `in` - Input file
  - `out` - Output file
- `Doc/` - Documentation directory
  - `division-of-labor.pdf` - Team responsibilities
  - `test` - Test documentation/files
- `Cargo.toml` - Rust project configuration and dependencies
- `Cargo.lock` - Locked dependency versions
- `input.txt` - Sample input file
- `output.txt` - Sample output file
- `.gitignore` - Git ignore rules

## How to Compile and Run

- [Development Log]

## Group Meetings

## Extra Credit
We have implemented all three optional Extra Credit functionalities
* Support unlimited number of pipes [2]
* Support piping and I/O redirection in a single command [2]
* Shell-ception: Execute your shell from within a running shell process repeatedly [1]
