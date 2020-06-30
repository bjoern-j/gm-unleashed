use std::iter::Peekable;
use itertools::Itertools;

pub type Tokens = Vec<Token>;
pub type Links = Vec<Link>;

#[derive(PartialEq, Eq, Debug)]
pub enum Token {
    Text(String),
    Asterisk,
    DoubleAsterisk,
    OpenSquareBrace,
    LinkMiddle,
    CloseRoundBrace,
    LineBreak,
}

pub struct Link {
    target : String,
}

impl Link {
    pub fn new(target : String) -> Self {
        Link {
            target,
        }
    }
    pub fn target(&self) -> &str { &self.target }
}

pub struct Markdown {
    pub text : Vec<String>,
    pub styles : Vec<StyleSpan>,
    pub breaks : Vec<Break>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct StyleSpan {
    pub span : Span,
    pub style : Style,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct Span {
    pub start : usize,
    pub end : usize,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Style {
    Italic,
    Bold,
    Link{ target : String },
}

#[derive(PartialEq, Eq, Debug)]
pub struct Break {
    pub pos : usize,
}

pub fn tokenize<S>(text : S) -> Tokens
    where S : Into<String>
{
    _tokenize(text.into().chars().peekable())
}

pub fn extract_links<'a, T>(tokens : T) -> Links 
    where T : IntoIterator<Item=&'a Token>
{
    _extract_links(tokens.into_iter())
}

pub fn parse(tokens : Tokens) -> Markdown {
    _parse(tokens.into_iter().peekable())
}

fn  _parse<T>(mut tokens : Peekable<T>) -> Markdown 
    where T : Iterator<Item=Token>
{
    let mut text = Vec::new();
    let mut styles = Vec::new();
    let mut breaks = Vec::new();
    let mut italic_span_start = None;
    let mut bold_span_start = None;
    let mut link_span_start = None;
    let mut link_span_end = 0;
    let mut inside_link_target = false;
    let mut link_target = String::new();
    let mut cur_idx = 0;
    loop {
        match tokens.next() {
            Some(Token::Text(txt)) => {
                if !inside_link_target {
                    text.push(txt);
                    cur_idx += 1;
                } else {
                    link_target.push_str(&txt);
                }
            }
            Some(Token::Asterisk) => {
                if let Some(span_start) = italic_span_start {
                    styles.push(StyleSpan{
                        style : Style::Italic,
                        span : Span { start : span_start, end : cur_idx - 1 },
                    });
                    italic_span_start = None;
                } else {
                    italic_span_start = Some(cur_idx);
                }
            },
            Some(Token::DoubleAsterisk) => {
                if let Some(span_start) = bold_span_start {
                    styles.push(StyleSpan{
                        style : Style::Bold,
                        span : Span { start : span_start, end : cur_idx - 1 },
                    });
                    bold_span_start = None;
                } else {
                    bold_span_start = Some(cur_idx);
                }                
            },
            Some(Token::OpenSquareBrace) => {
                if link_span_start.is_none() {
                    link_span_start = Some(cur_idx);
                }
            }
            Some(Token::LinkMiddle) => {
                link_span_end = cur_idx - 1;
                inside_link_target = true;
            }
            Some(Token::CloseRoundBrace) => {
                if inside_link_target {
                    styles.push(StyleSpan{
                        style : Style::Link{ target : link_target },
                        span : Span { start : link_span_start.unwrap(), end : link_span_end },
                    });    
                    link_target = String::new();                
                }
                inside_link_target = false;
            }
            Some(Token::LineBreak) => {
                breaks.push(Break{ pos : cur_idx });
            }
            None => { 
                break; 
            }
        }
    }
    Markdown {
        text,
        styles,
        breaks,
    }
}

fn _extract_links<'a, T>(tokens : T) -> Links
    where T : Iterator<Item=&'a Token>
{
    #[derive(PartialEq, Eq)]
    enum State { Initial, LinkOpened, TargetOpened }
    let mut links = Links::new();
    let mut state = State::Initial;
    let mut target = String::new();
    for token in tokens {
        if state == State::Initial && token == &Token::OpenSquareBrace {
            state = State::LinkOpened;
        }
        if state == State::LinkOpened && token == &Token::LinkMiddle {
            state = State::TargetOpened
        }
        if state == State::TargetOpened {
            if token == &Token::CloseRoundBrace {
                links.push(Link::new(target));
                target = String::new();
                state = State::Initial;
            } else if let Token::Text(text) = token {
                target.push_str(text);
            }
        }
    }
    links
}

fn _tokenize<Ch>(mut chars : Peekable<Ch>) -> Tokens 
    where Ch : Iterator<Item=char> + Clone
{
    let mut tokens = Tokens::new();
    loop {
        match chars.peek() {
            Some(&special_chars::ASTERISK) => {
                chars.next();
                if chars.peek() == Some(&special_chars::ASTERISK) {
                    tokens.push(Token::DoubleAsterisk);
                    chars.next();
                } else {
                    tokens.push(Token::Asterisk);
                }
            }
            Some(&special_chars::OPEN_SQUARE_BRACE) => {
                tokens.push(Token::OpenSquareBrace);
                chars.next();
            }
            Some(&special_chars::CLOSE_SQUARE_BRACE) => {
                chars.next();
                if chars.peek() == Some(&special_chars::OPEN_ROUND_BRACE) {
                    tokens.push(Token::LinkMiddle);
                    chars.next();
                } else {
                    tokens.push(Token::Text(special_chars::CLOSE_SQUARE_BRACE.to_string()));
                }
            }
            Some(&special_chars::CLOSE_ROUND_BRACE) => {
                tokens.push(Token::CloseRoundBrace);
                chars.next();
            }
            Some(&special_chars::OPEN_ROUND_BRACE) => {
                tokens.push(Token::Text(special_chars::OPEN_ROUND_BRACE.to_string()));
                chars.next();
            }
            Some(&special_chars::LINE_BREAK) => {
                tokens.push(Token::LineBreak);
                chars.next();
            }
            Some(_) => {
                let text = chars.take_while_ref(|ch|{ !is_special_char(ch) });
                let text_as_string : String = text.collect();
                tokens.push(Token::Text(text_as_string));
            }
            None => { break; }
        }
    }
    tokens
}

mod special_chars {
    pub const ASTERISK : char = '*';
    pub const OPEN_ROUND_BRACE : char = '(';
    pub const CLOSE_ROUND_BRACE : char = ')';
    pub const OPEN_SQUARE_BRACE : char = '[';
    pub const CLOSE_SQUARE_BRACE : char = ']';
    pub const LINE_BREAK : char = '\n';
}

fn is_special_char(&ch : &char) -> bool {
    ch == special_chars::ASTERISK ||
    ch == special_chars::OPEN_ROUND_BRACE || 
    ch == special_chars::CLOSE_ROUND_BRACE ||
    ch == special_chars::OPEN_SQUARE_BRACE ||
    ch == special_chars::CLOSE_SQUARE_BRACE ||
    ch == special_chars::LINE_BREAK
}

#[cfg(test)]
mod parser_tests {
    use super::*;
    const SAMPLE_TEXT : &str = "Some text";
    const LINK_TARGET : &str = "Linky Link";
    #[test]
    fn simple_text() {
        let md = parse(tokenize(SAMPLE_TEXT));
        assert_eq!(md.text[0], SAMPLE_TEXT);
        assert_eq!(md.text.len(), 1);
        assert_eq!(md.styles.len(), 0);
    }
    #[test]
    fn italic_text() {
        let md = parse(tokenize(format!("{} *{}* {}", SAMPLE_TEXT, SAMPLE_TEXT, SAMPLE_TEXT)));
        assert_eq!(md.text[0], format!("{} ", SAMPLE_TEXT));
        assert_eq!(md.text[1], SAMPLE_TEXT);
        assert_eq!(md.text[2], format!(" {}", SAMPLE_TEXT));
        assert_eq!(md.text.len(), 3);
        assert_eq!(md.styles[0], StyleSpan{ span : Span{ start : 1, end : 1 }, style : Style::Italic });
        assert_eq!(md.styles.len(), 1);
    }
    #[test]
    fn bold_text() {
        let md = parse(tokenize(format!("{} **{}** {}", SAMPLE_TEXT, SAMPLE_TEXT, SAMPLE_TEXT)));
        assert_eq!(md.text[0], format!("{} ", SAMPLE_TEXT));
        assert_eq!(md.text[1], SAMPLE_TEXT);
        assert_eq!(md.text[2], format!(" {}", SAMPLE_TEXT));
        assert_eq!(md.text.len(), 3);
        assert_eq!(md.styles[0], StyleSpan{ span : Span{ start : 1, end : 1 }, style : Style::Bold });
        assert_eq!(md.styles.len(), 1);
    }
    #[test]
    fn bold_and_italic_overlapping_text() {
        let md = parse(tokenize(format!("{}*{}**{}{}*{}**{}", SAMPLE_TEXT, SAMPLE_TEXT, SAMPLE_TEXT, SAMPLE_TEXT, SAMPLE_TEXT, SAMPLE_TEXT)));
        assert_eq!(md.text.len(), 5);
        assert_eq!(md.styles[0], StyleSpan{ span : Span{ start : 1, end : 2 }, style : Style::Italic });
        assert_eq!(md.styles[1], StyleSpan{ span : Span{ start : 2, end : 3 }, style : Style::Bold });
        assert_eq!(md.styles.len(), 2);
    }
    #[test]
    fn linked_text() {
        let md = parse(tokenize(format!("{}[{}]({}){}", SAMPLE_TEXT, SAMPLE_TEXT, LINK_TARGET, SAMPLE_TEXT)));
        assert_eq!(md.text.len(), 3);
        assert_eq!(md.styles[0], StyleSpan{ span : Span{ start : 1, end : 1 }, style : Style::Link{ target: LINK_TARGET.to_string()} });
        assert_eq!(md.styles.len(), 1);
    }
    #[test]
    fn line_break_text() {
        let md = parse(tokenize(format!("{}\n{}", SAMPLE_TEXT, SAMPLE_TEXT)));
        assert_eq!(md.text.len(), 2);
        assert_eq!(md.breaks[0].pos, 1);
        assert_eq!(md.breaks.len(), 1);
    }
    #[test]
    fn multiple_italic_spans() {
        let md = parse(tokenize(format!("{}*{}*{}*{}*{}*{}*{}", SAMPLE_TEXT, SAMPLE_TEXT, SAMPLE_TEXT, SAMPLE_TEXT, SAMPLE_TEXT, SAMPLE_TEXT, SAMPLE_TEXT)));
        assert_eq!(md.text.len(), 7);
        assert_eq!(md.styles[0], StyleSpan{ span : Span{ start : 1, end : 1 }, style : Style::Italic });
        assert_eq!(md.styles[1], StyleSpan{ span : Span{ start : 3, end : 3 }, style : Style::Italic });
        assert_eq!(md.styles[2], StyleSpan{ span : Span{ start : 5, end : 5 }, style : Style::Italic });                     
        assert_eq!(md.styles.len(), 3);
    }
}

#[cfg(test)]
mod link_extractor_tests {
    use super::*;
    const SAMPLE_TEXT : &str = "Some Text";
    const LINK_TARGET_1 : &str = "A link";
    const LINK_TARGET_2 : &str = "My link";
    #[test]
    fn no_links() {
        assert_eq!(extract_links(&tokenize(SAMPLE_TEXT)).len(), 0);
    }
    #[test]
    fn single_link() {
        let links = extract_links(&tokenize(format!("[{}]({})", SAMPLE_TEXT, LINK_TARGET_1)));
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target(), LINK_TARGET_1);
    }
    #[test]
    fn double_link() {
        let links = extract_links(&tokenize(format!("[{}]({}) blabla [{}]({})", SAMPLE_TEXT, LINK_TARGET_1, SAMPLE_TEXT, LINK_TARGET_2)));
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].target(), LINK_TARGET_1);        
        assert_eq!(links[1].target(), LINK_TARGET_2);
    }
    #[test]
    fn no_links_but_brackets() {
        assert_eq!(extract_links(&tokenize(format!("{} ({})", SAMPLE_TEXT, SAMPLE_TEXT))).len(), 0);
    }
}

#[cfg(test)]
mod tokenizer_tests {
    use super::*;
    const SAMPLE_TEXT : &str = "Some Text";
    #[test]
    fn plain_text() {
        let tokens = tokenize(SAMPLE_TEXT);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Text(SAMPLE_TEXT.to_string()));
    }
    #[test]
    fn italic_text() {
        let tokens = tokenize(format!("*{}*", SAMPLE_TEXT));
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], Token::Asterisk);
        assert_eq!(tokens[1], Token::Text(SAMPLE_TEXT.to_string()));
        assert_eq!(tokens[2], Token::Asterisk);
    }
    #[test]
    fn bold_text() {
        let tokens = tokenize(format!("**{}**", SAMPLE_TEXT));
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], Token::DoubleAsterisk);
        assert_eq!(tokens[1], Token::Text(SAMPLE_TEXT.to_string()));
        assert_eq!(tokens[2], Token::DoubleAsterisk);        
    }
    #[test]
    fn italic_bold_text() {
        let tokens = tokenize(format!("***{}***", SAMPLE_TEXT));
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0], Token::DoubleAsterisk);
        assert_eq!(tokens[1], Token::Asterisk);
        assert_eq!(tokens[2], Token::Text(SAMPLE_TEXT.to_string()));
        assert_eq!(tokens[3], Token::DoubleAsterisk);     
        assert_eq!(tokens[4], Token::Asterisk);
    }
    #[test]
    fn linked_text() {
        let tokens = tokenize(format!("[{}]({})", SAMPLE_TEXT, SAMPLE_TEXT));
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0], Token::OpenSquareBrace);
        assert_eq!(tokens[1], Token::Text(SAMPLE_TEXT.to_string()));
        assert_eq!(tokens[2], Token::LinkMiddle);
        assert_eq!(tokens[3], Token::Text(SAMPLE_TEXT.to_string()));
        assert_eq!(tokens[4], Token::CloseRoundBrace);
    }
    #[test]
    fn non_linked_text() {
        let tokens = tokenize(format!("[{}({})", SAMPLE_TEXT, SAMPLE_TEXT));        
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0], Token::OpenSquareBrace);
        assert_eq!(tokens[1], Token::Text(SAMPLE_TEXT.to_string()));
        assert_eq!(tokens[2], Token::Text(special_chars::OPEN_ROUND_BRACE.to_string()));
        assert_eq!(tokens[3], Token::Text(SAMPLE_TEXT.to_string()));
        assert_eq!(tokens[4], Token::CloseRoundBrace);        
    }
    #[test]
    fn line_broken_text() {
        let tokens = tokenize(format!("{}\n{}", SAMPLE_TEXT, SAMPLE_TEXT));
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], Token::Text(SAMPLE_TEXT.to_string()));
        assert_eq!(tokens[1], Token::LineBreak);
        assert_eq!(tokens[2], Token::Text(SAMPLE_TEXT.to_string()));
    }
}
