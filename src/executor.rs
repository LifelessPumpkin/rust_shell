use std::{env, ffi::CString, ptr};
use libc::dup2;
use nix::libc::{execv, fork, waitpid, pipe};
use std::path::Path;
use crate::parser::{Token, tokenize};

struct CommandPart {
    program: CString,
    args: Vec<CString>,
    direction: Option<Direction>,
    background: bool,
}

enum Direction {
    Pipe,
    RedirOut(String), // file name
    RedirIn(String),  // file name
    // maybe in future: AndThen, OrElse, Sequence
    
}

impl PartialEq for Direction {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // Compare only the discriminant (variant name) for Pipe
            (Direction::Pipe, Direction::Pipe) => true,
            // Compare RedirOut and RedirIn by their inner String values
            (Direction::RedirOut(s1), Direction::RedirOut(s2)) => s1 == s2,
            (Direction::RedirIn(s1), Direction::RedirIn(s2)) => s1 == s2,
            // All other combinations are not equal
            _ => false,
        }
    }
}

pub fn execute_command(command: &str) {
    // Phase 1: Tokenization and Expansion
    let tokens: Vec<_> = tokenize(command);
    let expanded_tokens: Vec<CString> = expand_tokens(tokens);

    // Phase 2: Interpretation and Execution
    let commands: Vec<CommandPart> = interpret_tokens(expanded_tokens);
    execute(commands);
}

fn expand_tokens(tokens: Vec<Token>) -> Vec<CString> {

	// •	When you see |, set the current CommandPart.direction = Some(Direction::Pipe)
	// •	When you see >, expect next token to be a filename → RedirOut(filename)
	// •	When you see <, same idea → RedirIn(filename)
	// •	When you see &, set background = true

    let mut expanded_tokens: Vec<CString> = Vec::new();
    let mut prev_token: Option<Token> = None;

    for token in tokens {
        match token {
            Token::EnvVar(name) => {
                if let Ok(val) = env::var(&name) {
                    expanded_tokens.push(CString::new(val).unwrap());
                    prev_token = Some(Token::EnvVar(name));
                } else {
                    expanded_tokens.push(CString::new(format!("Environment Variable Not Found")).unwrap());
                    prev_token = Some(Token::EnvVar(name));
                }
            }
            Token::Tilde => {
                let home = env::var("HOME").unwrap();
                expanded_tokens.push(CString::new(home).unwrap());
                prev_token = Some(Token::Tilde);
            }
            Token::Word(s) => {
                // If this is the first line or it follows a pipe, I need to search PATH for the executable
                // Otherwise, it's just an argument
                if prev_token.is_none() || prev_token == Some(Token::Pipe) {
                    // Search PATH for executable
                    let program = resolve_path(&s);
                    expanded_tokens.push(program);
                    prev_token = Some(Token::Word(s));
                    continue;
                } else {
                    let arg = CString::new(s).unwrap();
                    expanded_tokens.push(arg);
                    // Reset prev_token to indicate we've consumed the argument
                    prev_token = Some(Token::Word(String::new()));
                    continue;
                }

            }
            Token::Argument(s) => {
                let arg = CString::new(s).unwrap();
                expanded_tokens.push(arg);
                prev_token = Some(Token::Argument(String::new()));
            }
            Token::Pipe => {
                let pipe_token = CString::new("|").unwrap();
                expanded_tokens.push(pipe_token);
                prev_token = Some(Token::Pipe);
            }
            Token::RedirOut | Token::RedirIn | Token::Background => {
                // Skip for now — handle in phase 2
            }
        }
    }
    // For debugging purposes
    // println!("Expanded Tokens: ");
    // for t in expanded_tokens.iter() {
    //     println!("{} ", t.to_str().unwrap());
    // }
    // println!();
    expanded_tokens
}

fn interpret_tokens(tokens: Vec<CString>) -> Vec<CommandPart> {
    let mut command_parts: Vec<CommandPart> = Vec::new();
    let mut current_part: Option<CommandPart> = None;
    for t in tokens.iter() {
        // The first token is always the program
        // The rest are arguments until I hit a special token

        if current_part.is_none() {
            // if it's the first token, it must be the program
            current_part = Some(CommandPart {
                program: t.clone(),
                args: Vec::new(),
                direction: None,
                background: false,
            });
        } else {
            // if it's not the first token, it must be an argument
            current_part.as_mut().unwrap().args.push(t.clone());
        }

        if t.to_str().unwrap() == "|" {
            // In the previous if statement, I already added the pipe token as an argument
            // So I need to remove it from the args vector
            current_part.as_mut().unwrap().args.pop();
            // Add pipe direction to current part
            current_part.as_mut().unwrap().direction = Some(Direction::Pipe);
            // Push the current part to the vector
            command_parts.push(current_part.take().unwrap());
            // Start a new command part
            current_part = None;
            // execute_piped(cmds);
        } else if t.to_str().unwrap() == ">" {
            // Handle redirection
        } else if t.to_str().unwrap() == "<" {
            // Handle input redirection
        } else if t.to_str().unwrap() == "&" {
            // Handle background process
        } else {
            // Handle argument
            // If it's the last token and current_part is Some, push the current part
            if t == tokens.last().unwrap() && current_part.is_some() {
                command_parts.push(current_part.take().unwrap());
            }
        }
    }
    // For debugging purposes
    // println!("Interpreted Command Parts: ");
    // for part in command_parts.iter() {
    //     println!("Program: {}", part.program.to_str().unwrap());
    //     for arg in part.args.iter() {
    //         println!("Arg: {}", arg.to_str().unwrap());
    //     }
    //     if let Some(dir) = &part.direction {
    //         match dir {
    //             Direction::Pipe => println!("Direction: Pipe"),
    //             Direction::RedirOut(file) => println!("Direction: RedirOut to {}", file),
    //             Direction::RedirIn(file) => println!("Direction: RedirIn from {}", file),
    //         }
    //     } else {
    //         println!("Direction: None");
    //     }
    //     println!("Background: {}", part.background);
    // }
    command_parts
}

fn execute(command_parts: Vec<CommandPart>) {

    // If there's only one command part, execute it normally
    if command_parts.len() == 1 {
        unsafe {
            let pid: i32 = fork();
            if pid < 0 {
                eprintln!("Fork failed!");
            } else if pid == 0 { // Child process
                // Put the program in the args
                let mut argv = vec![command_parts[0].program.as_ptr()];

                // Add the rest of the args
                for arg in command_parts[0].args.iter() {
                    // println!("Arg: {}", CString::from_raw(*arg as *mut i8).to_str().unwrap());
                    argv.push(arg.as_ptr());
                }
                // Null terminate the args
                argv.push(ptr::null());
                execv(command_parts[0].program.as_ptr(), argv.as_ptr());
                // If execv returns, it must have failed
                eprintln!(
                    "execv failed for {}: {}",
                    command_parts[0].program.to_str().unwrap(),
                    std::io::Error::last_os_error()
                );
                std::process::exit(1);
            } else { // Parent process
                // To move things into bg i probably need to not wait here
                waitpid(pid, ptr::null_mut(), 0);
            }
        }
    } else {
        let mut previous_fd: Option<i32> = None; // the read-end of the previous pipe

        for part in command_parts.iter() {
            let mut pipe_fds: [i32; 2] = [0; 2];
            let use_pipe: bool = part.direction == Some(Direction::Pipe);

            // Create a pipe only if this command is piping to the next
            if use_pipe {
                unsafe {
                    if pipe(pipe_fds.as_mut_ptr()) == -1 {
                        panic!("pipe failed!");
                    }
                }
            }

            unsafe {
                let pid = fork();
                if pid < 0 {
                    panic!("fork failed!");
                } else if pid == 0 {
                    // CHILD

                    // If there was a previous pipe, set stdin to its read end
                    if let Some(fd) = previous_fd {
                        dup2(fd, libc::STDIN_FILENO);
                    }

                    // If we're piping to the next, set stdout to this pipe's write end
                    if use_pipe {
                        dup2(pipe_fds[1], libc::STDOUT_FILENO);
                    }

                    // Close unused fds in child
                    if let Some(fd) = previous_fd {
                        libc::close(fd);
                    }
                    if use_pipe {
                        libc::close(pipe_fds[0]);
                        libc::close(pipe_fds[1]);
                    }

                    // Prepare argv
                    let mut argv = vec![part.program.as_ptr()];
                    for arg in &part.args {
                        argv.push(arg.as_ptr());
                    }
                    argv.push(ptr::null());

                    execv(part.program.as_ptr(), argv.as_ptr());

                    eprintln!(
                        "execv failed for {}: {}",
                        part.program.to_str().unwrap(),
                        std::io::Error::last_os_error()
                    );
                    std::process::exit(1);
                } else {
                    // PARENT
                    if let Some(fd) = previous_fd {
                        libc::close(fd); // close previous pipe's read end
                    }
                    if use_pipe {
                        libc::close(pipe_fds[1]); // close write end in parent
                        previous_fd = Some(pipe_fds[0]); // keep read end for next iteration
                    } else {
                        previous_fd = None;
                    }
                    waitpid(pid, ptr::null_mut(), 0); // could be moved outside the loop if you want full parallel behavior
                }
            }
        }
    }
}

fn resolve_path(s: &str) -> CString {
    if let Ok(path) = env::var("PATH") {
        // Split the PATH variable into individual directories
        let paths: Vec<&str> = path.split(':').collect();

        // Iterate through the paths
        for path in paths.iter() {
            let full_path = &format!("{}/{}", path, s);

            // Check if the file exists and is executable
            if Path::new(&full_path).exists() {
                let program = CString::new(full_path.as_str()).unwrap();
                // expanded_tokens.push(program);

                // break; // Exit the loop if the executable is found and executed
                return program;
            }
        }
    } else {
        println!("PATH environment variable is not set.");
    }
    CString::new(s).unwrap() // Fallback to the original string if not found in PATH
}