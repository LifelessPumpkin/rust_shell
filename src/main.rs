use std::io::{stdin, stdout, Write};
use std::env;
use executor::execute_command;

mod executor;
mod parser;

fn main() {
    loop {
        create_prompt();

        let mut input: String = String::new();
        stdin().read_line(&mut input).unwrap();
        let command = input.trim();

        if command.is_empty() {
            continue;
        }
        if command == "exit" {
            break;
        }

        execute_command(command);
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

    let working_directory = match env::var(prompt[2]) {
        Ok(val) => val,
        Err(_) => String::new(),
    };

    print!("{}@{}:{}>",user, machine,working_directory);
    match stdout().flush() {
        Ok(res) => res,
        Err(_) => print!("Error Flushing")
    }    

}