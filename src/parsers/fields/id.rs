use std::borrow::Cow;

use crate::parsers::{header::HeaderValue, message_stream::MessageStream};

pub fn parse_id<'x>(stream: &'x MessageStream) -> HeaderValue<'x> {
    let mut token_start: usize = 0;
    let mut token_end: usize = 0;
    let mut is_token_safe = true;
    let mut is_id_part = false;
    let mut ids = Vec::new();

    while let Some(ch) = stream.next() {
        match ch {
            b'\n' => match stream.peek() {
                Some(b' ' | b'\t') => {
                    stream.advance(1);
                    continue;
                }
                _ => {
                    return match ids.len() {
                        1 => ids.pop().unwrap(),
                        0 => HeaderValue::Empty,
                        _ => HeaderValue::Array(ids),
                    };
                }
            },
            b'<' => {
                is_id_part = true;
                continue;
            }
            b'>' => {
                is_id_part = false;
                if token_start > 0 {
                    ids.push(HeaderValue::String(
                        stream
                            .get_string(token_start - 1, token_end, is_token_safe)
                            .unwrap(),
                    ));
                    is_token_safe = true;
                    token_start = 0;
                } else {
                    continue;
                }
            }
            b' ' | b'\t' | b'\r' => continue,
            0..=0x7f => (),
            _ => {
                if is_token_safe {
                    is_token_safe = false;
                }
            }
        }
        if is_id_part {
            if token_start == 0 {
                token_start = stream.get_pos();
            }
            token_end = stream.get_pos();
        }
    }
    HeaderValue::Empty
}

mod tests {
    use std::borrow::Cow;

    use crate::parsers::{
        fields::id::parse_id, header::HeaderValue, message_stream::MessageStream,
    };

    #[test]
    fn parse_message_ids() {
        let inputs = [
            (
                "<1234@local.machine.example>\n",
                vec!["1234@local.machine.example"],
            ),
            (
                "<1234@local.machine.example> <3456@example.net>\n",
                vec!["1234@local.machine.example", "3456@example.net"],
            ),
            (
                "<1234@local.machine.example>\n <3456@example.net> \n",
                vec!["1234@local.machine.example", "3456@example.net"],
            ),
            (
                "<1234@local.machine.example>\n\n <3456@example.net>\n",
                vec!["1234@local.machine.example"],
            ),
            (
                "              <testabcd.1234@silly.test>  \n",
                vec!["testabcd.1234@silly.test"],
            ),
            (
                "<5678.21-Nov-1997@example.com>\n",
                vec!["5678.21-Nov-1997@example.com"],
            ),
            (
                "<1234   @   local(blah)  .machine .example>\n",
                vec!["1234   @   local(blah)  .machine .example"],
            ),
        ];

        for input in inputs {
            assert_eq!(
                if input.1.len() == 1 {
                    HeaderValue::String((*input.1.first().unwrap()).into())
                } else {
                    HeaderValue::Array(
                        input
                            .1
                            .iter()
                            .map(|x| HeaderValue::String(Cow::Borrowed(x)))
                            .collect::<Vec<HeaderValue>>(),
                    )
                },
                parse_id(&MessageStream::new(input.0.as_bytes())),
                "Failed to parse '{}'",
                input.0
            );
        }
    }
}