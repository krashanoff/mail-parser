use super::{Decoder, DecoderResult};

enum QuotedPrintableState {
    None,
    Eq,
    Hex1,
}

pub struct QuotedPrintableDecoder {
    state: QuotedPrintableState,
    hex_1: i8,
}

impl QuotedPrintableDecoder {
    pub fn new() -> QuotedPrintableDecoder {
        QuotedPrintableDecoder {
            state: QuotedPrintableState::None,
            hex_1: 0,
        }
    }
}

impl Decoder for QuotedPrintableDecoder {
    fn ingest(&mut self, ch: u8) -> DecoderResult {
        match ch {
            b'=' => {
                // =
                if let QuotedPrintableState::None = self.state {
                    self.state = QuotedPrintableState::Eq;
                } else {
                    return DecoderResult::Error;
                }
            }
            b'\n' => {
                // \n
                match self.state {
                    QuotedPrintableState::Eq => {
                        self.state = QuotedPrintableState::None;
                    }
                    QuotedPrintableState::Hex1 => {
                        return DecoderResult::Error;
                    }
                    QuotedPrintableState::None => {
                        return DecoderResult::Byte(b'\n');
                    }
                }
            }
            b'\r' | b'\t' => (), // \r
            b' '..=b'~' => match self.state {
                QuotedPrintableState::None => {
                    return DecoderResult::Byte(ch);
                }
                QuotedPrintableState::Eq => {
                    self.hex_1 = unsafe { *HEX_MAP.get_unchecked(ch as usize) };
                    if self.hex_1 == -1 {
                        return DecoderResult::Error;
                    } else {
                        self.state = QuotedPrintableState::Hex1;
                    }
                }
                QuotedPrintableState::Hex1 => {
                    let hex2 = unsafe { *HEX_MAP.get_unchecked(ch as usize) };

                    self.state = QuotedPrintableState::None;
                    if hex2 == -1 {
                        return DecoderResult::Error;
                    } else {
                        return DecoderResult::Byte(((self.hex_1 as u8) << 4) | hex2 as u8);
                    }
                }
            },
            _ => {
                if let QuotedPrintableState::None = self.state {
                    return DecoderResult::Byte(b'!');
                } else {
                    return DecoderResult::Error;
                }
            }
        }

        DecoderResult::NeedData
    }
}

#[cfg(test)]
mod tests {
    use crate::decoders::{quoted_printable::QuotedPrintableDecoder, Decoder, DecoderResult};

    #[test]
    fn decode_success() {
        let input = [
            b"J'interdis aux marchands de vanter trop leurs marchandises. ".to_vec(),
            b"Car ils se font=\nvite p=C3=A9dagogues et t'enseignent comme but ce ".to_vec(),
            b"qui n'est par essence qu=\n'un moyen, et te trompant ainsi sur la route ".to_vec(),
            b"=C3=A0 suivre les voil=C3=\n=A0 bient=C3=B4t qui te d=C3=A9gradent, car ".to_vec(),
            b"si leur musique est vulgaire il=\ns te fabriquent pour te la vendre une ".to_vec(),
            b"=C3=A2me vulgaire.\n=E2=80=94=E2=80=89Antoine de Saint-Exup=C3=A9ry, ".to_vec(),
            b"Citadelle (1948)".to_vec(),
        ]
        .concat();
        let mut decoder = QuotedPrintableDecoder::new();
        let mut result = Vec::new();

        for ch in input {
            match decoder.ingest(ch) {
                DecoderResult::Byte(val) => {
                    result.push(val);
                }
                DecoderResult::NeedData => (),
                _ => {
                    assert!(false, "Error decoding.");
                }
            }
        }

        assert_eq!(
            std::str::from_utf8(&result[..]).unwrap(),
            concat!(
                "J'interdis aux marchands de vanter trop leurs marchandises. ",
                "Car ils se fontvite pédagogues et t'enseignent comme but ce qui ",
                "n'est par essence qu'un moyen, et te trompant ainsi sur la route ",
                "à suivre les voilà bientôt qui te dégradent, car si leur musique ",
                "est vulgaire ils te fabriquent pour te la vendre une âme vulgaire.\n",
                "— Antoine de Saint-Exupéry, Citadelle (1948)"
            )
        );
    }

    #[test]
    fn decode_invalid() {
        let inputs = [
            b"=2=123".to_vec(),
            b"= 20".to_vec(),
            b"=====".to_vec(),
            b"=20=20=XX".to_vec(),
            b"=AX".to_vec(),
            b"=\n=\n==".to_vec(),
            b"=\r=1z".to_vec(),
            b"=|".to_vec(),
        ];

        for input in inputs {
            let mut failed = false;
            let mut decoder = QuotedPrintableDecoder::new();

            for ch in &input {
                match decoder.ingest(*ch) {
                    DecoderResult::Byte(_) | DecoderResult::NeedData => (),
                    DecoderResult::ByteArray(_) => {
                        assert!(false, "Error decoding.");
                    }
                    DecoderResult::Error => {
                        failed = true;
                        break;
                    }
                }
            }

            assert_eq!(failed, true, "{}", std::str::from_utf8(&input[..]).unwrap());
        }
    }
}

/*
 * Adapted from Daniel Lemire's source:
 * https://github.com/lemire/Code-used-on-Daniel-Lemire-s-blog/blob/master/2019/04/17/hexparse.cpp
 *
 */

static HEX_MAP: &'static [i8] = &[
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, -1, -1, -1, -1, -1, -1, -1, 10, 11, 12, 13, 14, 15, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 10,
    11, 12, 13, 14, 15, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
];