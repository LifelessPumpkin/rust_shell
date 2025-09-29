## Project Documentation
### Division of Labor
#### Before
[Division of Labor](Doc/division-of-labor.pdf)

#### After
- Parts 1-8 -> Logan
- Part 9, testing, debugging -> Brooks
- Documentation -> Shawn

## File Structure
- `src/` - Source code dir
  - `main.rs` - Entry point of the program
  - `parser.rs` - Parses input job specifications
  - `job.rs` - Job data structure and related functions
  - `executor.rs` - Handles job execution logic
  - `builtins.rs` - Built-in command implementations
  - `job.rs.swo` - Swap file, temporary editor file
  - `in` - Input file
  - `out` - Output file
- `Doc/` - Documentation dir for uploading files for ease of use
  - `division-of-labor.pdf` - Team responsibilities
- `Cargo.toml` - Rust project configuration and dependencies
- `Cargo.lock` - dependency versions
- `input.txt` -  Sample input file
- `output.txt` - Sample output file
- `.gitignore` - Git ignore rules

## How to Compile and Run
### Prerequisites
- Rust toolchain (install from https://rustup.rs/)

### Compilation
```bash
cargo build --release
```
### Running The Program
```bash
cargo run --release

```

- [Development Log]

## Group Meetings
Our meetings mainly took place over the phone, and those meetings consisted of check-ins to talk about the project and our progress with our respective parts of the project. If anyone was struggling this was also the time to think about a solution all together and fix the issues.

## Extra Credit
We have implemented all three optional Extra Credit functionalities
* Support unlimited number of pipes [2]
* Support piping and I/O redirection in a single command [2]
* Shell-ception: Execute your shell from within a running shell process repeatedly [1]
