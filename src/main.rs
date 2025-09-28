use std::io::{stdin, stdout, Write};
use std::env;
use executor::execute_command_with_jobs;

mod executor;
mod parser;
mod builtins;
mod job;

use builtins::{try_builtin, BuiltinResult, History};
use job::JobTable;

fn main() {
    let mut jobs = JobTable::new();
    let mut history = History::new();

    loop {
        // poll for any completed background jobs before prompting
        jobs.poll();
        create_prompt();

        let mut input: String = String::new();
        stdin().read_line(&mut input).unwrap();
        let command = input.trim();

        if command.is_empty() {
            continue;
        }
        //if command == "exit" {
          //  break;
        //}

        //execute_command(command);
        // very simple tokenization for built-in detection only
        let simple_tokens: Vec<String> = command.split_whitespace().map(|s| s.to_string()).collect();
        match try_builtin(command, &simple_tokens, &mut jobs, &mut history) {
            BuiltinResult::Handled => { continue; }
            BuiltinResult::NotHandled => {
                // execute external / pipelines / redirections / background
                execute_command_with_jobs(command, &mut jobs);
                // treat as valid for history purposes
                history.push_valid(command);
            }
        }
    }
}

fn create_prompt() {

    let prompt = ["USER","MACHINE","PWD"];

    let user = match env::var(prompt[0]) {
        Ok(val) => val,
        Err(_) => String::new(),
    };

    let machine = match env::var(prompt[1]) {
        Ok(val) => val,
        Err(_) => String::new(),
    };

    let working_directory = match env::current_dir() {
        Ok(path) => path.display().to_string(),
        Err(_) => String::from("unknown"),
    };

    print!("{}@{}:{}>",user, machine,working_directory);
    match stdout().flush() {
        Ok(res) => res,
        Err(_) => print!("Error Flushing")
    }    

}
