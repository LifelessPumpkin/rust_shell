use std::ffi::CString;
use std::io::{stdin, stdout, Write};
use std::{env, ptr};
use executor::execute_command;
use parser::{tokenize, Token}; 

mod executor;
mod parser;

fn main() {
    loop {
        create_prompt();

        let mut input: String = String::new();
        stdin().read_line(&mut input).unwrap();

        let trimmed = input.trim();
        if trimmed.is_empty() {
            continue;
        }

        // only works when you put in the whole /path/to/program
        let program = CString::new(trimmed).unwrap();
        println!("Executing: {}", program.to_str().unwrap());
        // the program also needs to be in the args
        let arg0 = CString::new(trimmed).unwrap();
        // let arg1 = CString::new("-al").unwrap();

        let args = vec![arg0.as_ptr(), ptr::null()];

        let command: Vec<Token> = tokenize(trimmed);
        execute_command(command,program,args);

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