use std::borrow::Cow;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{anychar, char};
use nom::combinator::{map, peek, recognize};
use nom::error::ErrorKind;
use nom::multi::{fold_many1, many_till};
use nom::sequence::{delimited, pair};
use nom::IResult;

use crate::ast::{InterpolatedValue, Value};
use crate::lexer::ident;

/// Parse a quoted or interpolated string, starting and ending with the given `quote`.
pub fn string(quote: char) -> impl Fn(&str) -> IResult<&str, Value> {
    move |input: &str| {
        // Start quote
        let (input, _) = char(quote)(input)?;
        // First string part
        let (input, first_part) = string_part(quote)(input)?;

        // If the next char is an end-quote, this is a simple quoted string
        if let Ok((input, _)) = char::<_, (&str, ErrorKind)>(quote)(input) {
            return Ok((input, Value::QuotedString(first_part)));
        }

        // Otherwise try parsing an interpolated string
        interpolated_string_tail(quote, first_part)(input)
    }
}

/// Parse the literal part of a string.
///
/// Returns when the next chars would end the string or open an interpolation part.
fn string_part(quote: char) -> impl Fn(&str) -> IResult<&str, Cow<str>> {
    move |input: &str| {
        map(
            recognize(many_till(
                anychar,
                peek(alt((
                    tag("@{"),
                    tag("${"),
                    recognize(char(quote)),
                    // TODO: Handle escapes
                    // TODO: Handle newlines (and EOF?)
                ))),
            )),
            |s: &str| s.into(),
        )(input)
    }
}

/// Parse an interpolated variable/property in a string.
fn interpolated_part(input: &str) -> IResult<&str, InterpolatedValue> {
    alt((
        delimited(
            tag("@{"),
            map(ident, |name| InterpolatedValue::Variable(name)),
            tag("}"),
        ),
        delimited(
            tag("${"),
            map(ident, |name| InterpolatedValue::Property(name)),
            tag("}"),
        ),
    ))(input)
}

/// Parse the remainder of a string as an interpolated string.
fn interpolated_string_tail<'i>(
    quote: char,
    first_part: Cow<'i, str>,
) -> impl FnOnce(&'i str) -> IResult<&'i str, Value<'i>> {
    move |input: &'i str| {
        let (input, (strings, values)) = fold_many1(
            pair(interpolated_part, string_part(quote)),
            || (vec![first_part.clone()], vec![]),
            |mut acc, item| {
                acc.0.push(item.1);
                acc.1.push(item.0);
                acc
            },
        )(input)?;

        let (input, _) = char(quote)(input)?;

        Ok((input, Value::InterpolatedString(strings, values)))
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::{InterpolatedValue, Value};

    use super::string;

    #[test]
    fn test_string() {
        let cases = vec![
            // Quoted strings
            ("'test'", Ok(("", Value::QuotedString("test".into())))),
            // Interpolated strings
            (
                "'a @{b}'",
                Ok((
                    "",
                    Value::InterpolatedString(
                        vec!["a ".into(), "".into()],
                        vec![InterpolatedValue::Variable("b".into())],
                    ),
                )),
            ),
            (
                "'${a} b'",
                Ok((
                    "",
                    Value::InterpolatedString(
                        vec!["".into(), " b".into()],
                        vec![InterpolatedValue::Property("a".into())],
                    ),
                )),
            ),
        ];

        for (input, expected) in cases {
            assert_eq!(string('\'')(input), expected);
        }
    }
}
