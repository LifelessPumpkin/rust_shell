use std::{env, ffi::CString, ptr};
// use libc::waitpid;
use nix::libc::{execv, fork, waitpid};

use crate::parser::Token;

// mod parser;

pub fn execute_command(command: Vec<Token>, program: CString, args: Vec<*const i8>) {
    for t in command.iter() {
        match t {
            Token::Program(w) => println!("Program: {}", w),
            Token::Pipe => println!("Pipe"),
            Token::RedirOut => println!("Redirection Out"),
            Token::RedirIn => println!("Redirection In"),
            Token::EnvVar(v) => {
                match env::var(v) {
                    Ok(val) => println!("EnvVar: {}={}", v, val),
                    Err(_) => println!("EnvVar: {} not found", v),
                }
            },
            Token::Argument(a) => println!("Argument: {}", a),
            Token::Background => println!("Background"),
        }
    }

    // take in program and args from user input
    unsafe {
        let pid: i32 = fork();
        if pid < 0 {
            eprintln!("Fork failed!");
        } else if pid == 0 {
            // Child process
            execv(program.as_ptr(), args.as_ptr());
            // Only runs if execv fails
            eprintln!("execv failed!");
            std::process::exit(1);
        } else {
            // To move things into bg i probably need to not wait here
            // Parent process
            waitpid(pid, ptr::null_mut(), 0); 
            // println!("Child process {} finished", pid);
        }
    }

}