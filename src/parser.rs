pub enum Token {
    Program(String),
    Argument(String),
    Pipe,
    RedirOut,
    RedirIn,
    EnvVar(String),
    Background,
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
            p if p.starts_with('$') => tokens.push(Token::EnvVar(p[1..].to_string())),
            a if a.starts_with('-') => tokens.push(Token::Argument(part.to_string())),
            _ => tokens.push(Token::Program(part.to_string())),
        }
    }
    tokens    
}