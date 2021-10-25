use std::borrow::Cow;

use crate::{decoders::encoded_word::parse_encoded_word, parsers::message_stream::MessageStream};

struct ListParser<'x> {
    token_start: usize,
    token_end: usize,
    is_token_safe: bool,
    is_token_start: bool,
    tokens: Vec<Cow<'x, str>>,
    list: Vec<Cow<'x, str>>,
}

fn add_token<'x>(parser: &mut ListParser<'x>, stream: &MessageStream<'x>, add_space: bool) {
    if parser.token_start > 0 {
        if !parser.tokens.is_empty() {
            parser.tokens.push(" ".into());
        }
        parser.tokens.push(
            stream
                .get_string(
                    parser.token_start - 1,
                    parser.token_end,
                    parser.is_token_safe,
                )
                .unwrap(),
        );

        if add_space {
            parser.tokens.push(" ".into());
        }

        parser.token_start = 0;
        parser.is_token_safe = true;
        parser.is_token_start = true;
    }
}

fn add_tokens_to_list<'x>(parser: &mut ListParser<'x>) {
    if !parser.tokens.is_empty() {
        parser.list.push(if parser.tokens.len() == 1 {
            parser.tokens.pop().unwrap()
        } else {
            let value = parser.tokens.concat();
            parser.tokens.clear();
            value.into()
        });
    }
}

pub fn parse_comma_separared<'x>(stream: &MessageStream<'x>) -> Option<Vec<Cow<'x, str>>> {
    let mut parser = ListParser {
        token_start: 0,
        token_end: 0,
        is_token_safe: true,
        is_token_start: true,
        tokens: Vec::new(),
        list: Vec::new(),
    };

    while let Some(ch) = stream.next() {
        match ch {
            b'\n' => {
                add_token(&mut parser, stream, false);

                match stream.peek() {
                    Some(b' ' | b'\t') => {
                        if !parser.is_token_start {
                            parser.is_token_start = true;
                        }
                        stream.advance(1);
                        continue;
                    }
                    _ => {
                        add_tokens_to_list(&mut parser);
                        return if !parser.list.is_empty() {
                            parser.list.into()
                        } else {
                            None
                        };
                    }
                }
            }
            b' ' | b'\t' => {
                if !parser.is_token_start {
                    parser.is_token_start = true;
                }
                continue;
            }
            b'=' if parser.is_token_start => {
                if let Some(token) = parse_encoded_word(stream) {
                    add_token(&mut parser, stream, true);
                    parser.tokens.push(token.into());
                    continue;
                }
            }
            b',' => {
                add_token(&mut parser, stream, false);
                add_tokens_to_list(&mut parser);
                continue;
            }
            b'\r' => continue,
            0..=0x7f => (),
            _ => {
                if parser.is_token_safe {
                    parser.is_token_safe = false;
                }
            }
        }

        if parser.is_token_start {
            parser.is_token_start = false;
        }

        if parser.token_start == 0 {
            parser.token_start = stream.get_pos();
        }

        parser.token_end = stream.get_pos();
    }

    None
}

mod tests {
    use crate::parsers::{fields::list::parse_comma_separared, message_stream::MessageStream};

    #[test]
    fn parse_comma_separated_text() {
        let inputs = [
            (" one item  \n", vec!["one item"]),
            ("simple, list\n", vec!["simple".into(), "list".into()]),
            (
                "multi \r\n list, \r\n with, cr lf  \r\n",
                vec!["multi list".into(), "with".into(), "cr lf".into()],
            ),
            (
                "=?iso-8859-1?q?this is some text?=, in, a, list, \n",
                vec![
                    "this is some text".into(),
                    "in".into(),
                    "a".into(),
                    "list".into(),
                ],
            ),
            (
                concat!(
                    " =?ISO-8859-1?B?SWYgeW91IGNhbiByZWFkIHRoaXMgeW8=?=\n     ",
                    "=?ISO-8859-2?B?dSB1bmRlcnN0YW5kIHRoZSBleGFtcGxlLg==?=\n",
                    " , but, in a list, which, is, more, fun!\n"
                ),
                vec![
                    "If you can read this you understand the example.".into(),
                    "but".into(),
                    "in a list".into(),
                    "which".into(),
                    "is".into(),
                    "more".into(),
                    "fun!".into(),
                ],
            ),
            (
                "=?ISO-8859-1?Q?a?= =?ISO-8859-1?Q?b?=\n , listed\n",
                vec!["ab".into(), "listed".into()],
            ),
            (
                "ハロー・ワールド, and also, ascii terms\n",
                vec![
                    "ハロー・ワールド".into(),
                    "and also".into(),
                    "ascii terms".into(),
                ],
            ),
        ];

        for input in inputs {
            let mut str = input.0.to_string();
            assert_eq!(
                parse_comma_separared(&MessageStream::new(unsafe { str.as_bytes_mut() }),).unwrap(),
                input.1
            );
        }
    }
}
