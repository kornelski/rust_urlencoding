use std::str;
use std::string::FromUtf8Error;
use std::error::Error;
use std::fmt::{self, Display};
use std::io::Write;
use std::io;

/// Percent-encodes every byte except alphanumerics and `-`, `_`, `.`, `~`. Assumes UTF-8 encoding.
pub fn encode(data: &str) -> String {
    let mut escaped = Vec::with_capacity(data.len());
    encode_into(data, &mut escaped).unwrap();
    // Encoded string is guaranteed to be ASCII
    unsafe {
        String::from_utf8_unchecked(escaped)
    }
}

/// Precent-encodes every byte except alphanumerics and `-`, `_`, `.`, `~`.
/// Writes everything to a mutable string.
/// ```
/// # use urlencoding::encode_to_mut_string;
/// let mut s = "https://rust-lang.org?message=".to_string();
/// encode_to_mut_string("👾 Exterminate!", &mut s);
/// 
/// assert_eq!(s, "https://rust-lang.org?message=%F0%9F%91%BE%20Exterminate%21");
/// ```
pub fn encode_to_mut_string(data: &str, escaped: &mut String) {
    escaped.reserve(data.len());
    // Encoded string is guaranteed to be ASCII
    let escaped = unsafe { escaped.as_mut_vec() };
    encode_into(data, escaped).unwrap();
}

/// Writes the escaped data to an Writer.
#[inline]
fn encode_into<W: Write>(data: &str, mut escaped: W) -> io::Result<()> {
    for byte in data.as_bytes().iter() {
        match *byte {
            b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' |  b'-' | b'.' | b'_' | b'~' => {
                escaped.write_all(std::slice::from_ref(byte))?;
            },
            other => {
                escaped.write_all(&[b'%', to_hex_digit(other >> 4), to_hex_digit(other & 15)])?;
            },
        }
    }
    Ok(())
}

#[inline]
fn from_hex_digit(digit: u8) -> Option<u8> {
    match digit {
        b'0'..=b'9' => Some(digit - b'0'),
        b'A'..=b'F' => Some(digit - b'A' + 10),
        b'a'..=b'f' => Some(digit - b'a' + 10),
        _ => None,
    }
}

#[inline]
fn to_hex_digit(digit: u8) -> u8 {
    match digit {
        0..=9 => b'0' + digit,
        10..=255 => b'A' - 10 + digit,
    }
}

/// Decode percent-encoded string assuming UTF-8 encoding.
///
/// Unencoded `+` is preserved literally, and _not_ changed to a space.
pub fn decode(string: &str) -> Result<String, FromUrlEncodingError> {
    let mut out: Vec<u8> = Vec::with_capacity(string.len());
    let mut bytes = string.as_bytes().iter().copied();
    while let Some(b) = bytes.next() {
        match b {
            b'%' => {
                match bytes.next() {
                    Some(first) => match from_hex_digit(first) {
                        Some(first_val) => match bytes.next() {
                            Some(second) => match from_hex_digit(second) {
                                Some(second_val) => {
                                    out.push((first_val << 4) | second_val);
                                },
                                None => {
                                    out.push(b'%');
                                    out.push(first);
                                    out.push(second);
                                },
                            },
                            None => {
                                out.push(b'%');
                                out.push(first);
                            },
                        },
                        None => {
                            out.push(b'%');
                            out.push(first);
                        },
                    },
                    None => out.push(b'%'),
                };
            },
            other => out.push(other),
        }
    }
    String::from_utf8(out).map_err(|error| FromUrlEncodingError::Utf8CharacterError {error})
}

/// Error when decoding invalid UTF-8
#[derive(Debug)]
pub enum FromUrlEncodingError {
    /// Not used. Exists for backwards-compatibility only
    UriCharacterError { character: char, index: usize },
    /// Percent-encoded string contained bytes that can't be expressed in UTF-8
    Utf8CharacterError { error: FromUtf8Error },
}

impl Error for FromUrlEncodingError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            FromUrlEncodingError::UriCharacterError {character: _, index: _} => None,
            FromUrlEncodingError::Utf8CharacterError {error} => Some(error)
        }
    }
}

impl Display for FromUrlEncodingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            FromUrlEncodingError::UriCharacterError {character, index} =>
                write!(f, "invalid URI char [{}] at [{}]", character, index),
            FromUrlEncodingError::Utf8CharacterError {error} =>
                write!(f, "invalid utf8 char: {}", error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{encode, encode_to_mut_string};
    use super::decode;
    use super::from_hex_digit;

    #[test]
    fn it_encodes_successfully() {
        let expected = "this%20that";
        assert_eq!(expected, encode("this that"));
    }

    #[test]
    fn it_encodes_successfully_emoji() {
        let emoji_string = "👾 Exterminate!";
        let expected = "%F0%9F%91%BE%20Exterminate%21";
        assert_eq!(expected, encode(emoji_string));
    }

    #[test]
    fn it_encodes_to_a_mut_string() {
        let emoji_string = "👾 Exterminate!";
        let expected = "%F0%9F%91%BE%20Exterminate%21";
        let mut s = String::new();
        encode_to_mut_string(emoji_string, &mut s);
        assert_eq!(expected, s);
    }

    #[test]
    fn it_decodes_successfully() {
        let expected = String::from("this that");
        let encoded = "this%20that";
        assert_eq!(expected, decode(encoded).unwrap());
    }

    #[test]
    fn it_decodes_successfully_emoji() {
        let expected = String::from("👾 Exterminate!");
        let encoded = "%F0%9F%91%BE%20Exterminate%21";
        assert_eq!(expected, decode(encoded).unwrap());
    }

    #[test]
    fn it_decodes_unsuccessfully_emoji() {
        let bad_encoded_string = "👾 Exterminate!";

        assert_eq!(bad_encoded_string, decode(bad_encoded_string).unwrap());
    }


    #[test]
    fn misc() {
        assert_eq!(3, from_hex_digit(b'3').unwrap());
        assert_eq!(10, from_hex_digit(b'a').unwrap());
        assert_eq!(15, from_hex_digit(b'F').unwrap());
        assert_eq!(None, from_hex_digit(b'G'));
        assert_eq!(None, from_hex_digit(9));

        assert_eq!("pureascii", encode("pureascii"));
        assert_eq!("pureascii", decode("pureascii").unwrap());
        assert_eq!("", encode(""));
        assert_eq!("", decode("").unwrap());
        assert_eq!("%00", encode("\0"));
        assert_eq!("\0", decode("\0").unwrap());
        assert!(decode("%F0%0F%91%BE%20Hello%21").is_err());
        assert_eq!("this%2that", decode("this%2that").unwrap());
        assert_eq!("this that", decode("this%20that").unwrap());
        assert_eq!("this that%", decode("this%20that%").unwrap());
        assert_eq!("this that%2", decode("this%20that%2").unwrap());
    }
}
