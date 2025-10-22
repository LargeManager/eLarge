use logos::Logos;

use ratatui::{
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
};

#[derive(Logos, Debug, PartialEq)]
pub enum Token {
    // --- Keywords ---
    #[token("fn")]
    Fn,
    #[token("let")]
    Let,
    #[token("return")]
    Return,

    // --- Types ---
    #[token("i32")]
    TypeI32,
    #[token("u64")]
    TypeU64,
    #[token("bool")]
    TypeBool,
    #[token("String")]
    TypeString,

    // --- Punctuation and Operators ---
    #[token("(")]
    OpenParen,
    #[token(")")]
    CloseParen,
    #[token("{")]
    OpenBrace,
    #[token("}")]
    CloseBrace,
    #[token("->")]
    Arrow,
    #[token(",")]
    Comma,
    #[token(":")]
    Colon,
    #[token(";")]
    Semicolon,
    #[token("=")]
    Assign,
    #[token("+")]
    Plus,

    // --- Identifiers and Literals ---
    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Ident,
    #[regex("[0-9]+")]
    IntegerLiteral,
    #[regex(r#""[^"]*""#)]
    StringLiteral,

    // --- Other Content ---
    #[regex(r"[ \t\f]+")]
    Whitespace,
    #[token("\n")]
    Newline,
}

fn token_to_style(token: &Token) -> Style {
    match token {
        Token::Let | Token::Return => Style::default().fg(Color::LightBlue).bold(),
        Token::Fn | Token::TypeI32 | Token::TypeU64 | Token::TypeBool | Token::TypeString => {
            Style::default()
                .fg(Color::Cyan)
                .underline_color(Color::Yellow)
        }
        Token::IntegerLiteral => Style::default().fg(Color::LightYellow),
        Token::StringLiteral => Style::default().fg(Color::Yellow),
        Token::Ident => Style::default().fg(Color::Red).underline_color(Color::Red),
        Token::Semicolon | Token::Comma | Token::Colon => Style::default().fg(Color::DarkGray),
        Token::Assign | Token::Plus | Token::Arrow => Style::default().fg(Color::Magenta),
        Token::Whitespace | Token::Newline => Style::default().fg(Color::Green),
        Token::CloseParen | Token::OpenParen => Style::default().fg(Color::Green),
        Token::OpenBrace | Token::CloseBrace => Style::default().fg(Color::LightBlue),
    }
}

pub fn lex_and_style(text: &str) -> Text<'_> {
    let lexer = Token::lexer(text);
    let mut lines: Vec<Line> = Vec::new();
    let mut current_spans: Vec<Span> = Vec::new();

    for (token, span_range) in lexer.spanned() {
        let slice = &text[span_range.clone()];

        let style = if let Ok(x) = token {
            token_to_style(&x)
        } else {
            Style::new()
        };

        let mut start = 0;
        for (idx, ch) in slice.char_indices() {
            if ch == '\n' {
                // piece includes up to and including this newline
                let piece = &slice[start..=idx];
                // remove the trailing newline for visual span content
                let content = &piece[..piece.len() - 1];
                if !content.is_empty() {
                    current_spans.push(Span::styled(content.to_string(), style));
                }
                // finalize current line and start a new one
                lines.push(Line::from(current_spans));
                current_spans = Vec::new();
                start = idx + ch.len_utf8();
            }
        }

        if start < slice.len() {
            let trailing = &slice[start..];
            if !trailing.is_empty() {
                current_spans.push(Span::styled(trailing.to_string(), style));
            }
        } else {
        }
    }
    if !current_spans.is_empty() {
        lines.push(Line::from(current_spans));
    } else if lines.is_empty() {
        lines.push(Line::from(Vec::<Span>::new()));
    }
    Text::from(lines)
}
