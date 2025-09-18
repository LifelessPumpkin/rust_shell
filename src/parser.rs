use std::{env, ffi::CString, path::Path};

pub enum Token {
    Word(String),
    Argument(String),
    EnvVar(String),
    Pipe,
    RedirOut,
    RedirIn,
    Background,
    Tilde
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // Compare only the discriminant (variant name) for Pipe
            (Token::Pipe, Token::Pipe) => true,
            // Compare RedirOut and RedirIn by their inner String values
            // (Token::RedirOut(s1), Token::RedirOut(s2)) => s1 == s2,
            // (Token::RedirIn(s1), Token::RedirIn(s2)) => s1 == s2,
            // All other combinations are not equal
            _ => false,
        }
    }
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    // Split using whitespace, then match on special chars
    for part in input.split_whitespace() {
        match part {
            "|" => tokens.push(Token::Pipe),
            ">" => tokens.push(Token::RedirOut),
            "<" => tokens.push(Token::RedirIn),
            "&" => tokens.push(Token::Background),
            // Only need to handle the ~ and ~/ case
            t if t.starts_with('~') || t.starts_with("~/") => tokens.push(Token::Tilde), 
            p if p.starts_with('$') => tokens.push(Token::EnvVar(p[1..].to_string())),
            a if a.starts_with('-') => tokens.push(Token::Argument(part.to_string())),
            _ => tokens.push(Token::Word(part.to_string())),
        }
    }
    tokens
}

pub fn expand_tokens(tokens: Vec<Token>) -> Vec<CString> {
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
                // If it follows a redirection, it's a filename, so just add it as an argument
                if prev_token.is_none() || prev_token == Some(Token::Pipe) {
                    // Search PATH for executable
                    let program = resolve_path(&s);
                    expanded_tokens.push(program);
                    prev_token = Some(Token::Word(s));
                    continue;
                } else if prev_token == Some(Token::RedirOut) || prev_token == Some(Token::RedirIn) {
                    // It's a filename for redirection
                    let filename = CString::new(s).unwrap();
                    expanded_tokens.push(filename);
                    prev_token = Some(Token::Word(String::new()));
                    continue;
                    
                } 
                else {
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
            Token::RedirOut => {
                // If it's a redirection, the next token should be a filename
                expanded_tokens.push(CString::new(">").unwrap());
                prev_token = Some(Token::RedirOut);
            }
            Token::RedirIn => {
                // If it's a redirection, the next token should be a filename
                expanded_tokens.push(CString::new("<").unwrap());
                prev_token = Some(Token::RedirIn);
            }
            
            Token::Background => {
                let bg_token = CString::new("&").unwrap();
                expanded_tokens.push(bg_token);
                prev_token = Some(Token::Background);
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

                // Exit the loop if the executable is found and executed
                return program;
            }
        }
    } else {
        println!("PATH environment variable is not set.");
    }
    CString::new(s).unwrap() // Fallback to the original string if not found in PATH
}