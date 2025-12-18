use std::str::Chars;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn merge(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    // Keywords
    Nutrient,
    Ingredient,
    Formula,
    Import,
    Template,
    Min,
    Max,
    As,

    // Literals
    Ident,
    Number,
    String,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Dot,
    Colon,
    Comma,

    // Delimiters
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    LParen,
    RParen,

    // Special
    Newline,
    Comment,
    Eof,
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub text: String,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span, text: impl Into<String>) -> Self {
        Self {
            kind,
            span,
            text: text.into(),
        }
    }
}

pub struct Lexer<'a> {
    source: &'a str,
    chars: Chars<'a>,
    pos: usize,
    current: Option<char>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut chars = source.chars();
        let current = chars.next();
        Self {
            source,
            chars,
            pos: 0,
            current,
        }
    }

    pub fn tokenize(source: &str) -> Vec<Token> {
        let mut lexer = Lexer::new(source);
        let mut tokens = Vec::new();
        loop {
            let token = lexer.next_token();
            let is_eof = token.kind == TokenKind::Eof;
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        tokens
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.current;
        self.current = self.chars.next();
        if c.is_some() {
            self.pos += c.unwrap().len_utf8();
        }
        c
    }

    fn peek(&self) -> Option<char> {
        self.current
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c == ' ' || c == '\t' || c == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_line_comment(&mut self) -> Token {
        let start = self.pos;
        self.advance(); // first /
        self.advance(); // second /
        while let Some(c) = self.peek() {
            if c == '\n' {
                break;
            }
            self.advance();
        }
        Token::new(
            TokenKind::Comment,
            Span::new(start, self.pos),
            &self.source[start..self.pos],
        )
    }

    fn skip_block_comment(&mut self) -> Token {
        let start = self.pos;
        self.advance(); // /
        self.advance(); // *
        loop {
            match self.peek() {
                Some('*') => {
                    self.advance();
                    if self.peek() == Some('/') {
                        self.advance();
                        break;
                    }
                }
                Some(_) => {
                    self.advance();
                }
                None => break, // Unterminated comment
            }
        }
        Token::new(
            TokenKind::Comment,
            Span::new(start, self.pos),
            &self.source[start..self.pos],
        )
    }

    fn read_string(&mut self) -> Token {
        let start = self.pos;
        self.advance(); // opening quote
        while let Some(c) = self.peek() {
            if c == '"' {
                self.advance();
                break;
            }
            if c == '\n' {
                // Unterminated string
                break;
            }
            self.advance();
        }
        Token::new(
            TokenKind::String,
            Span::new(start, self.pos),
            &self.source[start..self.pos],
        )
    }

    fn read_number(&mut self) -> Token {
        let start = self.pos;

        // Optional negative
        if self.peek() == Some('-') {
            self.advance();
        }

        // Integer part
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                self.advance();
            } else {
                break;
            }
        }

        // Decimal part
        if self.peek() == Some('.') {
            // Look ahead to see if it's a decimal or a dot operator
            let mut chars = self.chars.clone();
            if let Some(next) = chars.next() {
                if next.is_ascii_digit() {
                    self.advance(); // consume the dot
                    while let Some(c) = self.peek() {
                        if c.is_ascii_digit() {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
            }
        }

        Token::new(
            TokenKind::Number,
            Span::new(start, self.pos),
            &self.source[start..self.pos],
        )
    }

    fn read_ident(&mut self) -> Token {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                self.advance();
            } else {
                break;
            }
        }
        let text = &self.source[start..self.pos];
        let kind = match text {
            "nutrient" => TokenKind::Nutrient,
            "ingredient" => TokenKind::Ingredient,
            "formula" => TokenKind::Formula,
            "import" => TokenKind::Import,
            "template" => TokenKind::Template,
            "min" => TokenKind::Min,
            "max" => TokenKind::Max,
            "as" => TokenKind::As,
            _ => TokenKind::Ident,
        };
        Token::new(kind, Span::new(start, self.pos), text)
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let start = self.pos;

        let Some(c) = self.peek() else {
            return Token::new(TokenKind::Eof, Span::new(start, start), "");
        };

        match c {
            '\n' => {
                self.advance();
                Token::new(TokenKind::Newline, Span::new(start, self.pos), "\n")
            }
            '/' => {
                let mut chars = self.chars.clone();
                match chars.next() {
                    Some('/') => self.skip_line_comment(),
                    Some('*') => self.skip_block_comment(),
                    _ => {
                        self.advance();
                        Token::new(TokenKind::Slash, Span::new(start, self.pos), "/")
                    }
                }
            }
            '"' => self.read_string(),
            '+' => {
                self.advance();
                Token::new(TokenKind::Plus, Span::new(start, self.pos), "+")
            }
            '-' => {
                // Could be negative number or minus operator
                let mut chars = self.chars.clone();
                if let Some(next) = chars.next() {
                    if next.is_ascii_digit() {
                        return self.read_number();
                    }
                }
                self.advance();
                Token::new(TokenKind::Minus, Span::new(start, self.pos), "-")
            }
            '*' => {
                self.advance();
                Token::new(TokenKind::Star, Span::new(start, self.pos), "*")
            }
            '%' => {
                self.advance();
                Token::new(TokenKind::Percent, Span::new(start, self.pos), "%")
            }
            '.' => {
                self.advance();
                Token::new(TokenKind::Dot, Span::new(start, self.pos), ".")
            }
            ':' => {
                self.advance();
                Token::new(TokenKind::Colon, Span::new(start, self.pos), ":")
            }
            ',' => {
                self.advance();
                Token::new(TokenKind::Comma, Span::new(start, self.pos), ",")
            }
            '{' => {
                self.advance();
                Token::new(TokenKind::LBrace, Span::new(start, self.pos), "{")
            }
            '}' => {
                self.advance();
                Token::new(TokenKind::RBrace, Span::new(start, self.pos), "}")
            }
            '[' => {
                self.advance();
                Token::new(TokenKind::LBracket, Span::new(start, self.pos), "[")
            }
            ']' => {
                self.advance();
                Token::new(TokenKind::RBracket, Span::new(start, self.pos), "]")
            }
            '(' => {
                self.advance();
                Token::new(TokenKind::LParen, Span::new(start, self.pos), "(")
            }
            ')' => {
                self.advance();
                Token::new(TokenKind::RParen, Span::new(start, self.pos), ")")
            }
            c if c.is_ascii_digit() => self.read_number(),
            c if c.is_alphabetic() || c == '_' => self.read_ident(),
            _ => {
                self.advance();
                Token::new(
                    TokenKind::Error,
                    Span::new(start, self.pos),
                    &self.source[start..self.pos],
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keywords() {
        let tokens = Lexer::tokenize("nutrient ingredient formula import min max");
        let kinds: Vec<_> = tokens.iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Nutrient,
                TokenKind::Ingredient,
                TokenKind::Formula,
                TokenKind::Import,
                TokenKind::Min,
                TokenKind::Max,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_numbers() {
        let tokens = Lexer::tokenize("100 8.5 -20 0.005");
        let texts: Vec<_> = tokens.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(texts, vec!["100", "8.5", "-20", "0.005", ""]);
    }

    #[test]
    fn test_string() {
        let tokens = Lexer::tokenize("\"Hello World\"");
        assert_eq!(tokens[0].kind, TokenKind::String);
        assert_eq!(tokens[0].text, "\"Hello World\"");
    }

    #[test]
    fn test_operators() {
        let tokens = Lexer::tokenize("+ - * / % . : ,");
        let kinds: Vec<_> = tokens.iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Plus,
                TokenKind::Minus,
                TokenKind::Star,
                TokenKind::Slash,
                TokenKind::Percent,
                TokenKind::Dot,
                TokenKind::Colon,
                TokenKind::Comma,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_comments() {
        let tokens = Lexer::tokenize("foo // comment\nbar");
        let kinds: Vec<_> = tokens.iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Ident,
                TokenKind::Comment,
                TokenKind::Newline,
                TokenKind::Ident,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_formula_snippet() {
        let source = r#"nutrient protein {
  name: "Crude Protein"
}"#;
        let tokens = Lexer::tokenize(source);
        let kinds: Vec<_> = tokens
            .iter()
            .filter(|t| t.kind != TokenKind::Newline)
            .map(|t| t.kind)
            .collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Nutrient,
                TokenKind::Ident, // protein
                TokenKind::LBrace,
                TokenKind::Ident, // name
                TokenKind::Colon,
                TokenKind::String,
                TokenKind::RBrace,
                TokenKind::Eof,
            ]
        );
    }
}
