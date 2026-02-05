use logos::Logos;
use std::fmt;

/// Span represents a range in the source code
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

/// Token with its span information
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

/// All token types in Apex
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r\n\f]+")]
#[logos(skip r"//[^\n]*")]
#[logos(skip r"/\*[^*]*\*+(?:[^/*][^*]*\*+)*/")]
pub enum TokenKind {
    // Keywords - Access Modifiers
    #[token("public", ignore(ascii_case))]
    Public,
    #[token("private", ignore(ascii_case))]
    Private,
    #[token("protected", ignore(ascii_case))]
    Protected,
    #[token("global", ignore(ascii_case))]
    Global,

    // Keywords - Class/Type Modifiers
    #[token("abstract", ignore(ascii_case))]
    Abstract,
    #[token("virtual", ignore(ascii_case))]
    Virtual,
    #[token("override", ignore(ascii_case))]
    Override,
    #[token("static", ignore(ascii_case))]
    Static,
    #[token("final", ignore(ascii_case))]
    Final,
    #[token("transient", ignore(ascii_case))]
    Transient,
    #[token("with sharing", ignore(ascii_case))]
    WithSharing,
    #[token("without sharing", ignore(ascii_case))]
    WithoutSharing,
    #[token("inherited sharing", ignore(ascii_case))]
    InheritedSharing,

    // Keywords - Type Declarations
    #[token("class", ignore(ascii_case))]
    Class,
    #[token("interface", ignore(ascii_case))]
    Interface,
    #[token("enum", ignore(ascii_case))]
    Enum,
    #[token("trigger", ignore(ascii_case))]
    Trigger,

    // Keywords - Control Flow
    #[token("if", ignore(ascii_case))]
    If,
    #[token("else", ignore(ascii_case))]
    Else,
    #[token("for", ignore(ascii_case))]
    For,
    #[token("while", ignore(ascii_case))]
    While,
    #[token("do", ignore(ascii_case))]
    Do,
    #[token("switch", ignore(ascii_case))]
    Switch,
    #[token("when", ignore(ascii_case))]
    When,
    #[token("break", ignore(ascii_case))]
    Break,
    #[token("continue", ignore(ascii_case))]
    Continue,
    #[token("return", ignore(ascii_case))]
    Return,
    #[token("throw", ignore(ascii_case))]
    Throw,
    #[token("try", ignore(ascii_case))]
    Try,
    #[token("catch", ignore(ascii_case))]
    Catch,
    #[token("finally", ignore(ascii_case))]
    Finally,

    // Keywords - OOP
    #[token("extends", ignore(ascii_case))]
    Extends,
    #[token("implements", ignore(ascii_case))]
    Implements,
    #[token("this", ignore(ascii_case))]
    This,
    #[token("super", ignore(ascii_case))]
    Super,
    #[token("new", ignore(ascii_case))]
    New,
    #[token("instanceof", ignore(ascii_case))]
    Instanceof,

    // Keywords - Primitive Types
    #[token("void", ignore(ascii_case))]
    Void,
    #[token("boolean", ignore(ascii_case))]
    Boolean,
    #[token("integer", ignore(ascii_case))]
    Integer,
    #[token("long", ignore(ascii_case))]
    Long,
    #[token("double", ignore(ascii_case))]
    Double,
    #[token("decimal", ignore(ascii_case))]
    Decimal,
    #[token("string", ignore(ascii_case))]
    StringType,
    #[token("blob", ignore(ascii_case))]
    Blob,
    #[token("date", ignore(ascii_case))]
    Date,
    #[token("datetime", ignore(ascii_case))]
    Datetime,
    #[token("time", ignore(ascii_case))]
    Time,
    #[token("id", ignore(ascii_case))]
    Id,
    #[token("object", ignore(ascii_case))]
    Object,

    // Keywords - Collection Types
    #[token("list", ignore(ascii_case))]
    List,
    #[token("map", ignore(ascii_case))]
    Map,

    // Keywords - SOQL/SOSL
    #[token("select", ignore(ascii_case))]
    Select,
    #[token("from", ignore(ascii_case))]
    From,
    #[token("where", ignore(ascii_case))]
    Where,
    #[token("find", ignore(ascii_case))]
    Find,
    #[token("returning", ignore(ascii_case))]
    Returning,
    #[token("order", ignore(ascii_case))]
    Order,
    #[token("by", ignore(ascii_case))]
    By,
    #[token("limit", ignore(ascii_case))]
    Limit,
    #[token("offset", ignore(ascii_case))]
    Offset,
    #[token("asc", ignore(ascii_case))]
    Asc,
    #[token("desc", ignore(ascii_case))]
    Desc,
    #[token("nulls", ignore(ascii_case))]
    Nulls,
    #[token("first", ignore(ascii_case))]
    First,
    #[token("last", ignore(ascii_case))]
    Last,
    #[token("group", ignore(ascii_case))]
    Group,
    #[token("having", ignore(ascii_case))]
    Having,
    #[token("and", ignore(ascii_case))]
    And,
    #[token("or", ignore(ascii_case))]
    Or,
    #[token("not", ignore(ascii_case))]
    Not,
    #[token("in", ignore(ascii_case))]
    In,
    #[token("like", ignore(ascii_case))]
    Like,
    #[token("includes", ignore(ascii_case))]
    Includes,
    #[token("excludes", ignore(ascii_case))]
    Excludes,

    // Keywords - DML
    #[token("insert", ignore(ascii_case))]
    Insert,
    #[token("update", ignore(ascii_case))]
    Update,
    #[token("upsert", ignore(ascii_case))]
    Upsert,
    #[token("delete", ignore(ascii_case))]
    Delete,
    #[token("undelete", ignore(ascii_case))]
    Undelete,
    #[token("merge", ignore(ascii_case))]
    Merge,

    // Keywords - Trigger Events
    #[token("before", ignore(ascii_case))]
    Before,
    #[token("after", ignore(ascii_case))]
    After,
    #[token("on", ignore(ascii_case))]
    On,

    // Keywords - Other
    #[token("null", ignore(ascii_case))]
    Null,
    #[token("true", ignore(ascii_case))]
    True,
    #[token("false", ignore(ascii_case))]
    False,
    #[token("get", ignore(ascii_case))]
    Get,
    // "set" is used both as collection type and property accessor
    // Context will determine which meaning applies - parser handles this
    #[token("set", ignore(ascii_case))]
    Set,
    #[token("testmethod", ignore(ascii_case))]
    TestMethod,
    #[token("webservice", ignore(ascii_case))]
    WebService,

    // Operators - Arithmetic
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,

    // Operators - Comparison
    #[token("==")]
    EqEq,
    #[token("!=")]
    NotEq,
    #[token("<>")]
    LtGt,
    #[token("===")]
    EqEqEq,
    #[token("!==")]
    NotEqEq,
    #[token("<")]
    Lt,
    #[token(">")]
    Gt,
    #[token("<=")]
    LtEq,
    #[token(">=")]
    GtEq,

    // Operators - Logical
    #[token("&&")]
    AndAnd,
    #[token("||")]
    OrOr,
    #[token("!")]
    Bang,

    // Operators - Bitwise
    #[token("&")]
    Amp,
    #[token("|")]
    Pipe,
    #[token("^")]
    Caret,
    #[token("~")]
    Tilde,
    #[token("<<")]
    LtLt,
    #[token(">>")]
    GtGt,
    #[token(">>>")]
    GtGtGt,

    // Operators - Assignment
    #[token("=")]
    Eq,
    #[token("+=")]
    PlusEq,
    #[token("-=")]
    MinusEq,
    #[token("*=")]
    StarEq,
    #[token("/=")]
    SlashEq,
    #[token("&=")]
    AmpEq,
    #[token("|=")]
    PipeEq,
    #[token("^=")]
    CaretEq,
    #[token("%=")]
    PercentEq,
    #[token("<<=")]
    LtLtEq,
    #[token(">>=")]
    GtGtEq,
    #[token(">>>=")]
    GtGtGtEq,

    // Operators - Increment/Decrement
    #[token("++")]
    PlusPlus,
    #[token("--")]
    MinusMinus,

    // Operators - Other
    #[token("?")]
    Question,
    #[token("?.")]
    QuestionDot,
    #[token("??")]
    QuestionQuestion,
    #[token("=>")]
    Arrow,

    // Delimiters
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token(";")]
    Semicolon,
    #[token(",")]
    Comma,
    #[token(".")]
    Dot,
    #[token(":")]
    Colon,
    #[token("@")]
    At,

    // Literals - Numeric
    // Hex literals: 0x or 0X followed by hex digits
    #[regex(r"0[xX][0-9a-fA-F]+", parse_hex)]
    HexLiteral(i64),

    // Binary literals: 0b or 0B followed by binary digits
    #[regex(r"0[bB][01]+", parse_binary)]
    BinaryLiteral(i64),

    // Octal literals: 0 followed by octal digits (but not just 0)
    #[regex(r"0[0-7]+", parse_octal)]
    OctalLiteral(i64),

    // Regular integer (must come after hex/binary/octal to avoid conflicts)
    #[regex(r"[0-9]+", priority = 1, callback = |lex| lex.slice().parse::<i64>().ok())]
    IntegerLiteral(i64),

    #[regex(r"[0-9]+[lL]", parse_long)]
    LongLiteral(i64),

    // Hex long literals
    #[regex(r"0[xX][0-9a-fA-F]+[lL]", parse_hex_long)]
    HexLongLiteral(i64),

    #[regex(r"[0-9]+\.[0-9]+([eE][+-]?[0-9]+)?", |lex| lex.slice().parse::<f64>().ok())]
    DoubleLiteral(f64),

    #[regex(r"'([^'\\]|\\.)*'", parse_string)]
    StringLiteral(String),

    // Identifiers
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),

    // Annotation
    #[regex(r"@[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice()[1..].to_string())]
    Annotation(String),

    // End of file
    Eof,
}

fn parse_string(lex: &mut logos::Lexer<TokenKind>) -> Option<String> {
    let slice = lex.slice();
    // Remove surrounding quotes and unescape
    let inner = &slice[1..slice.len() - 1];
    let mut result = String::new();
    let mut chars = inner.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some('\\') => result.push('\\'),
                Some('\'') => result.push('\''),
                Some('"') => result.push('"'),
                Some(c) => {
                    result.push('\\');
                    result.push(c);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }
    Some(result)
}

fn parse_long(lex: &mut logos::Lexer<TokenKind>) -> Option<i64> {
    let slice = lex.slice();
    slice[..slice.len() - 1].parse::<i64>().ok()
}

fn parse_hex(lex: &mut logos::Lexer<TokenKind>) -> Option<i64> {
    let slice = lex.slice();
    // Skip "0x" or "0X" prefix
    i64::from_str_radix(&slice[2..], 16).ok()
}

fn parse_hex_long(lex: &mut logos::Lexer<TokenKind>) -> Option<i64> {
    let slice = lex.slice();
    // Skip "0x" or "0X" prefix and trailing L/l
    i64::from_str_radix(&slice[2..slice.len() - 1], 16).ok()
}

fn parse_binary(lex: &mut logos::Lexer<TokenKind>) -> Option<i64> {
    let slice = lex.slice();
    // Skip "0b" or "0B" prefix
    i64::from_str_radix(&slice[2..], 2).ok()
}

fn parse_octal(lex: &mut logos::Lexer<TokenKind>) -> Option<i64> {
    let slice = lex.slice();
    // Skip leading "0"
    i64::from_str_radix(&slice[1..], 8).ok()
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Public => write!(f, "public"),
            TokenKind::Private => write!(f, "private"),
            TokenKind::Protected => write!(f, "protected"),
            TokenKind::Global => write!(f, "global"),
            TokenKind::Abstract => write!(f, "abstract"),
            TokenKind::Virtual => write!(f, "virtual"),
            TokenKind::Override => write!(f, "override"),
            TokenKind::Static => write!(f, "static"),
            TokenKind::Final => write!(f, "final"),
            TokenKind::Transient => write!(f, "transient"),
            TokenKind::WithSharing => write!(f, "with sharing"),
            TokenKind::WithoutSharing => write!(f, "without sharing"),
            TokenKind::InheritedSharing => write!(f, "inherited sharing"),
            TokenKind::Class => write!(f, "class"),
            TokenKind::Interface => write!(f, "interface"),
            TokenKind::Enum => write!(f, "enum"),
            TokenKind::Trigger => write!(f, "trigger"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::For => write!(f, "for"),
            TokenKind::While => write!(f, "while"),
            TokenKind::Do => write!(f, "do"),
            TokenKind::Switch => write!(f, "switch"),
            TokenKind::When => write!(f, "when"),
            TokenKind::Break => write!(f, "break"),
            TokenKind::Continue => write!(f, "continue"),
            TokenKind::Return => write!(f, "return"),
            TokenKind::Throw => write!(f, "throw"),
            TokenKind::Try => write!(f, "try"),
            TokenKind::Catch => write!(f, "catch"),
            TokenKind::Finally => write!(f, "finally"),
            TokenKind::Extends => write!(f, "extends"),
            TokenKind::Implements => write!(f, "implements"),
            TokenKind::This => write!(f, "this"),
            TokenKind::Super => write!(f, "super"),
            TokenKind::New => write!(f, "new"),
            TokenKind::Instanceof => write!(f, "instanceof"),
            TokenKind::Null => write!(f, "null"),
            TokenKind::True => write!(f, "true"),
            TokenKind::False => write!(f, "false"),
            TokenKind::Identifier(s) => write!(f, "{}", s),
            TokenKind::IntegerLiteral(n) => write!(f, "{}", n),
            TokenKind::LongLiteral(n) => write!(f, "{}L", n),
            TokenKind::HexLiteral(n) => write!(f, "0x{:X}", n),
            TokenKind::HexLongLiteral(n) => write!(f, "0x{:X}L", n),
            TokenKind::BinaryLiteral(n) => write!(f, "0b{:b}", n),
            TokenKind::OctalLiteral(n) => write!(f, "0{:o}", n),
            TokenKind::DoubleLiteral(n) => write!(f, "{}", n),
            TokenKind::StringLiteral(s) => write!(f, "'{}'", s),
            TokenKind::Eof => write!(f, "EOF"),
            _ => write!(f, "{:?}", self),
        }
    }
}

/// Lexer for Apex source code
pub struct Lexer<'a> {
    inner: logos::Lexer<'a, TokenKind>,
    peeked: Option<Token>,
    peeked2: Option<Token>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            inner: TokenKind::lexer(source),
            peeked: None,
            peeked2: None,
        }
    }

    fn read_next(&mut self) -> Token {
        match self.inner.next() {
            Some(Ok(kind)) => {
                let span = self.inner.span();
                Token::new(kind, Span::new(span.start, span.end))
            }
            Some(Err(_)) => {
                // Skip invalid token and try next
                self.read_next()
            }
            None => Token::new(TokenKind::Eof, Span::new(0, 0)),
        }
    }

    pub fn next_token(&mut self) -> Token {
        if let Some(token) = self.peeked.take() {
            self.peeked = self.peeked2.take();
            return token;
        }
        self.read_next()
    }

    pub fn peek(&mut self) -> &Token {
        if self.peeked.is_none() {
            self.peeked = Some(self.read_next());
        }
        self.peeked.as_ref().unwrap()
    }

    /// Peek at the second token ahead (the one after peek())
    pub fn peek_second(&mut self) -> &Token {
        // Ensure first peek is filled
        if self.peeked.is_none() {
            self.peeked = Some(self.read_next());
        }
        // Ensure second peek is filled
        if self.peeked2.is_none() {
            self.peeked2 = Some(self.read_next());
        }
        self.peeked2.as_ref().unwrap()
    }

    pub fn source(&self) -> &'a str {
        self.inner.source()
    }
}

/// Tokenize an entire source string into a vector of tokens
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keywords() {
        let tokens = tokenize("public class MyClass");
        assert_eq!(tokens[0].kind, TokenKind::Public);
        assert_eq!(tokens[1].kind, TokenKind::Class);
        assert!(matches!(tokens[2].kind, TokenKind::Identifier(ref s) if s == "MyClass"));
    }

    #[test]
    fn test_case_insensitive() {
        let tokens = tokenize("PUBLIC CLASS PRIVATE");
        assert_eq!(tokens[0].kind, TokenKind::Public);
        assert_eq!(tokens[1].kind, TokenKind::Class);
        assert_eq!(tokens[2].kind, TokenKind::Private);
    }

    #[test]
    fn test_string_literal() {
        let tokens = tokenize("'hello world'");
        assert!(matches!(&tokens[0].kind, TokenKind::StringLiteral(s) if s == "hello world"));
    }

    #[test]
    fn test_string_escape() {
        let tokens = tokenize(r"'hello\nworld'");
        assert!(matches!(&tokens[0].kind, TokenKind::StringLiteral(s) if s == "hello\nworld"));
    }

    #[test]
    fn test_numbers() {
        let tokens = tokenize("42 100L 3.14");
        assert!(matches!(tokens[0].kind, TokenKind::IntegerLiteral(42)));
        assert!(matches!(tokens[1].kind, TokenKind::LongLiteral(100)));
        assert!(matches!(tokens[2].kind, TokenKind::DoubleLiteral(n) if (n - 3.14).abs() < 0.001));
    }

    #[test]
    fn test_operators() {
        let tokens = tokenize("+ - * / == != < > <= >=");
        assert_eq!(tokens[0].kind, TokenKind::Plus);
        assert_eq!(tokens[1].kind, TokenKind::Minus);
        assert_eq!(tokens[2].kind, TokenKind::Star);
        assert_eq!(tokens[3].kind, TokenKind::Slash);
        assert_eq!(tokens[4].kind, TokenKind::EqEq);
        assert_eq!(tokens[5].kind, TokenKind::NotEq);
        assert_eq!(tokens[6].kind, TokenKind::Lt);
        assert_eq!(tokens[7].kind, TokenKind::Gt);
        assert_eq!(tokens[8].kind, TokenKind::LtEq);
        assert_eq!(tokens[9].kind, TokenKind::GtEq);
    }

    #[test]
    fn test_comments() {
        let tokens = tokenize("public // this is a comment\nclass");
        assert_eq!(tokens[0].kind, TokenKind::Public);
        assert_eq!(tokens[1].kind, TokenKind::Class);
    }

    #[test]
    fn test_multiline_comment() {
        let tokens = tokenize("public /* this is\na multiline\ncomment */ class");
        assert_eq!(tokens[0].kind, TokenKind::Public);
        assert_eq!(tokens[1].kind, TokenKind::Class);
    }

    #[test]
    fn test_annotation() {
        let tokens = tokenize("@isTest public class");
        assert!(matches!(&tokens[0].kind, TokenKind::Annotation(s) if s == "isTest"));
        assert_eq!(tokens[1].kind, TokenKind::Public);
    }
}
