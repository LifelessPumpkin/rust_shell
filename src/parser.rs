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
    // Expect only the first part to be a command, rest are args
    // Unless they start with special chars
    for part in input.split_whitespace() {
        // let is_first = tokens.is_empty();
        // if is_first && (part == "|" || part == ">" || part == "<" || part == "&") {
        //     // First token cannot be a special char
        //     continue;
        // }

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