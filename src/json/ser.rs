use crate::ser::{Fragment, Map, Seq, Serialize};
use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

/// Serialize any serializable type into a JSON string.
///
/// ```rust
/// use miniserde::{json, Serialize};
///
/// #[derive(Serialize, Debug)]
/// struct Example {
///     code: u32,
///     message: String,
/// }
///
/// fn main() {
///     let example = Example {
///         code: 200,
///         message: "reminiscent of Serde".to_owned(),
///     };
///
///     let j = json::to_string(&example);
///     println!("{}", j);
/// }
/// ```
pub fn to_string<T>(value: &T) -> String
where
    T: ?Sized + Serialize,
{
    to_string_impl(&value)
}

struct Serializer<'a> {
    stack: Vec<Layer<'a>>,
}

enum Layer<'a> {
    Seq(Box<dyn Seq + 'a>),
    Map(Box<dyn Map + 'a>),
}

impl<'a> Drop for Serializer<'a> {
    fn drop(&mut self) {
        // Drop layers in reverse order.
        while !self.stack.is_empty() {
            self.stack.pop();
        }
    }
}

fn to_string_impl(value: &dyn Serialize) -> String {
    let mut out = String::new();
    let mut serializer = Serializer { stack: Vec::new() };
    let mut fragment = value.begin();

    loop {
        match fragment {
            Fragment::Null => out.push_str("null"),
            Fragment::Bool(b) => out.push_str(if b { "true" } else { "false" }),
            Fragment::Str(s) => escape_str(&s, &mut out),
            Fragment::U64(n) => out.push_str(itoa::Buffer::new().format(n)),
            Fragment::I64(n) => out.push_str(itoa::Buffer::new().format(n)),
            Fragment::F64(n) => {
                if n.is_finite() {
                    out.push_str(ryu::Buffer::new().format_finite(n));
                } else {
                    out.push_str("null");
                }
            }
            Fragment::Seq(mut seq) => {
                out.push('[');
                // invariant: `seq` must outlive `first`
                match unsafe { extend_lifetime!(seq.next() as Option<&dyn Serialize>) } {
                    Some(first) => {
                        serializer.stack.push(Layer::Seq(seq));
                        fragment = first.begin();
                        continue;
                    }
                    None => out.push(']'),
                }
            }
            Fragment::Map(mut map) => {
                out.push('{');
                // invariant: `map` must outlive `first`
                match unsafe { extend_lifetime!(map.next() as Option<(Cow<str>, &dyn Serialize)>) }
                {
                    Some((key, first)) => {
                        escape_str(&key, &mut out);
                        out.push(':');
                        serializer.stack.push(Layer::Map(map));
                        fragment = first.begin();
                        continue;
                    }
                    None => out.push('}'),
                }
            }
        }

        loop {
            match serializer.stack.last_mut() {
                Some(Layer::Seq(seq)) => {
                    // invariant: `seq` must outlive `next`
                    match unsafe { extend_lifetime!(seq.next() as Option<&dyn Serialize>) } {
                        Some(next) => {
                            out.push(',');
                            fragment = next.begin();
                            break;
                        }
                        None => out.push(']'),
                    }
                }
                Some(Layer::Map(map)) => {
                    // invariant: `map` must outlive `next`
                    match unsafe {
                        extend_lifetime!(map.next() as Option<(Cow<str>, &dyn Serialize)>)
                    } {
                        Some((key, next)) => {
                            out.push(',');
                            escape_str(&key, &mut out);
                            out.push(':');
                            fragment = next.begin();
                            break;
                        }
                        None => out.push('}'),
                    }
                }
                None => return out,
            }
            serializer.stack.pop();
        }
    }
}

// Clippy false positive: https://github.com/rust-lang/rust-clippy/issues/5169
#[allow(clippy::zero_prefixed_literal)]
fn escape_str(value: &str, out: &mut String) {
    out.push('"');

    let bytes = value.as_bytes();
    let mut start = 0;

    for (i, &byte) in bytes.iter().enumerate() {
        let escape = ESCAPE[byte as usize];
        if escape == 0 {
            continue;
        }

        if start < i {
            out.push_str(&value[start..i]);
        }

        match escape {
            self::BB => out.push_str("\\b"),
            self::TT => out.push_str("\\t"),
            self::NN => out.push_str("\\n"),
            self::FF => out.push_str("\\f"),
            self::RR => out.push_str("\\r"),
            self::QU => out.push_str("\\\""),
            self::BS => out.push_str("\\\\"),
            self::U => {
                static HEX_DIGITS: [u8; 16] = *b"0123456789abcdef";
                out.push_str("\\u00");
                out.push(HEX_DIGITS[(byte >> 4) as usize] as char);
                out.push(HEX_DIGITS[(byte & 0xF) as usize] as char);
            }
            _ => unreachable!(),
        }

        start = i + 1;
    }

    if start != bytes.len() {
        out.push_str(&value[start..]);
    }

    out.push('"');
}

const BB: u8 = b'b'; // \x08
const TT: u8 = b't'; // \x09
const NN: u8 = b'n'; // \x0A
const FF: u8 = b'f'; // \x0C
const RR: u8 = b'r'; // \x0D
const QU: u8 = b'"'; // \x22
const BS: u8 = b'\\'; // \x5C
const U: u8 = b'u'; // \x00...\x1F except the ones above

// Lookup table of escape sequences. A value of b'x' at index i means that byte
// i is escaped as "\x" in JSON. A value of 0 means that byte i is not escaped.
#[rustfmt::skip]
static ESCAPE: [u8; 256] = [
    //  1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
    U,  U,  U,  U,  U,  U,  U,  U, BB, TT, NN,  U, FF, RR,  U,  U, // 0
    U,  U,  U,  U,  U,  U,  U,  U,  U,  U,  U,  U,  U,  U,  U,  U, // 1
    0,  0, QU,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 2
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 3
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 4
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, BS,  0,  0,  0, // 5
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 6
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 7
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 8
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 9
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // A
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // B
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // C
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // D
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // E
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // F
];
