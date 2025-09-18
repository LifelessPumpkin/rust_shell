use std::{ffi::CString, ptr};
use libc::{close, dup2, open, O_CREAT, O_RDONLY, O_TRUNC, O_WRONLY, STDIN_FILENO, STDOUT_FILENO};
use nix::libc::{execv, fork, waitpid, pipe};
use crate::parser::{expand_tokens, tokenize};

struct CommandPart {
    program: CString,
    args: Vec<CString>,
    redir_in: Option<String>,   // e.g. < input.txt
    redir_out: Option<String>,  // e.g. > output.txt
    direction: Option<Direction>, // still keep this for pipes or future chaining
    background: bool,
}

enum Direction {
    Pipe,
    // RedirOut(String), // file name
    // RedirIn(String),  // file name
    // maybe in future: AndThen, OrElse, Sequence
    
}

impl PartialEq for Direction {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // Compare only the discriminant (variant name) for Pipe
            (Direction::Pipe, Direction::Pipe) => true,
            // Compare RedirOut and RedirIn by their inner String values
            // (Direction::RedirOut(s1), Direction::RedirOut(s2)) => s1 == s2,
            // (Direction::RedirIn(s1), Direction::RedirIn(s2)) => s1 == s2,
            // All other combinations are not equal
            // _ => false,
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

fn interpret_tokens(tokens: Vec<CString>) -> Vec<CommandPart> {
    let mut command_parts: Vec<CommandPart> = Vec::new();
    let mut current_part: Option<CommandPart> = None;

    let mut tokens_iter = tokens.iter().peekable();
    while let Some(t) = tokens_iter.next() {
        // The first token is always the program
        // The rest are arguments until I hit a special token

        if current_part.is_none() {
            // if it's the first token, it must be the program
            current_part = Some(CommandPart {
                program: t.clone(),
                args: Vec::new(),
                redir_in: None,
                redir_out: None,
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
            current_part.as_mut().unwrap().args.pop(); // remove ">" from args
            if let Some(next_token) = tokens_iter.next() {
                let filename = next_token.to_str().unwrap().to_string();
                current_part.as_mut().unwrap().redir_out = Some(filename);
            }
        }
        else if t.to_str().unwrap() == "<" {
            current_part.as_mut().unwrap().args.pop(); // remove "<" from args
            if let Some(next_token) = tokens_iter.next() {
                let filename = next_token.to_str().unwrap().to_string();
                current_part.as_mut().unwrap().redir_in = Some(filename);
            }
        } else if t.to_str().unwrap() == "&" {
            current_part.as_mut().unwrap().background = true;
            // Remove the & from args
            current_part.as_mut().unwrap().args.pop();
            // Push the current part to the vector
            command_parts.push(current_part.take().unwrap());
            // Start a new command part
            current_part = None;
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
                if !command_parts[0].background {
                    waitpid(pid, ptr::null_mut(), 0);
                }
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
                if let Some(filename) = &part.redir_out {
                    let file = CString::new(filename.clone()).unwrap();
                    let fd = open(file.as_ptr(), O_WRONLY | O_CREAT | O_TRUNC, 0o644);
                    if fd == -1 {
                        panic!("open for redir out failed!");
                    }
                    dup2(fd, STDOUT_FILENO);
                    close(fd);
                }

                if let Some(filename) = &part.redir_in {
                    let file = CString::new(filename.clone()).unwrap();
                    let fd = open(file.as_ptr(), O_RDONLY);
                    if fd == -1 {
                        panic!("open for redir in failed!");
                    }
                    dup2(fd, STDIN_FILENO);
                    close(fd);
                }

                    // If there was a previous pipe, set stdin to its read end
                    if let Some(fd) = previous_fd {
                        dup2(fd, STDIN_FILENO);
                    }

                    // If we're piping to the next, set stdout to this pipe's write end
                    if use_pipe {
                        dup2(pipe_fds[1], STDOUT_FILENO);
                    }

                    // Close unused fds in child
                    if let Some(fd) = previous_fd {
                        close(fd);
                    }
                    if use_pipe {
                        close(pipe_fds[0]);
                        close(pipe_fds[1]);
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
                    if !part.background {
                        waitpid(pid, ptr::null_mut(), 0);
                    }
                }
            }
        }
    }
}