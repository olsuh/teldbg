//#![cfg(feature = "alloc")]

use std::num::ParseIntError;

use nom::{
    branch::alt,
    bytes::complete::{escaped, tag},
    character::complete::{
        alpha1, alphanumeric1, anychar, char, digit1, multispace0, multispace1, none_of, one_of,
    },
    combinator::{cut, map, map_res, recognize, value},
    error::{context, ContextError, FromExternalError, ParseError, VerboseError},
    //error::{context, convert_error, ContextError, ErrorKind, ParseError, VerboseError},
    multi::{many0, many0_count, many1, separated_list0},
    sequence::{pair, preceded, terminated},
    IResult
};

/*
/// parser combinators are constructed from the bottom up:
/// first we write parsers for the smallest elements (here a space character),
/// then we'll combine them in larger parsers
fn sp<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    let chars = " \t\r\n";

    // nom combinators like `take_while` return a function. That function is the
    // parser,to which we can pass the input
    take_while(move |c| chars.contains(c))(i)
}*/

/// A nom parser has the following signature:
/// `Input -> IResult<Input, Output, Error>`, with `IResult` defined as:
/// `type IResult<I, O, E = (I, ErrorKind)> = Result<(I, O), Err<E>>;`
///
/// most of the times you can ignore the error type and use the default (but this
/// examples shows custom error types later on!)
///
/// Here we use `&str` as input type, but nom parsers can be generic over
/// the input type, and work directly with `&[u8]` or any other type that
/// implements the required traits.
///
/// Finally, we can see here that the input and output type are both `&str`
/// with the same lifetime tag. This means that the produced value is a subslice
/// of the input data. and there is no allocation needed. This is the main idea
/// behind nom's performance.
fn parse_str<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    escaped(character, '\\', one_of("\"n\\"))(i)
}

/// `tag(string)` generates a parser that recognizes the argument string.
///
/// we can combine it with other functions, like `value` that takes another
/// parser, and if that parser returns without an error, returns a given
/// constant value.
///
/// `alt` is another combinator that tries multiple parsers one by one, until
/// one of them succeeds
fn boolean<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, bool, E> {
    // This is a parser that returns `true` if it sees the string "true", and
    // an error otherwise
    let parse_true = value(true, tag("true"));

    // This is a parser that returns `false` if it sees the string "false", and
    // an error otherwise
    let parse_false = value(false, tag("false"));

    // `alt` combines the two parsers. It returns the result of the first
    // successful parser, or an error
    alt((parse_true, parse_false))(input)
}

fn null<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, (), E> {
    value((), alt((tag(""), tag("null"))))(input)
}
fn dec_u32<'a, E: ParseError<&'a str> + FromExternalError<&'a str, ParseIntError>>(
    input: &'a str,
) -> IResult<&'a str, Int, E> {
    map_res(digit1, str::parse)(input)
}

/// this parser combines the previous `parse_str` parser, that recognizes the
/// interior of a string, with a parse to recognize the double quote character,
/// before the string (using `preceded`) and after the string (using `terminated`).
///
/// `context` and `cut` are related to error management:
/// - `cut` transforms an `Err::Error(e)` in `Err::Failure(e)`, signaling to
/// combinators like  `alt` that they should not try other parsers. We were in the
/// right branch (since we found the `"` character) but encountered an error when
/// parsing the string
/// - `context` lets you add a static string to provide more information in the
/// error chain (to indicate which parser had an error)
fn string<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    context(
        "string",
        preceded(char('\"'), cut(terminated(parse_str, char('\"')))),
    )(i)
}

/*
fn u16_hex(input: &str) -> IResult<&str, u16> {
    map_res(take(4usize), |s| u16::from_str_radix(s, 16))(input)
  }

  fn unicode_escape(input: &str) -> IResult<&str, char> {
    map_opt(
      alt((
        // Not a surrogate
        map(verify(u16_hex, |cp| !(0xD800..0xE000).contains(cp)), |cp| {
          cp as u32
        }),
        // See https://en.wikipedia.org/wiki/UTF-16#Code_points_from_U+010000_to_U+10FFFF for details
        map(
          verify(
            separated_pair(u16_hex, tag("\\u"), u16_hex),
            |(high, low)| (0xD800..0xDC00).contains(high) && (0xDC00..0xE000).contains(low),
          ),
          |(high, low)| {
            let high_ten = (high as u32) - 0xD800;
            let low_ten = (low as u32) - 0xDC00;
            (high_ten << 10) + low_ten + 0x10000
          },
        ),
      )),
      // Could be probably replaced with .unwrap() or _unchecked due to the verify checks
      std::char::from_u32,
    )(input)
  }
 */

/*
  fn string(input: &str) -> IResult<&str, String> {
    delimited(
      char('"'),
      fold_many0(character, String::new, |mut string, c| {
        string.push(c);
        string
      }),
      char('"'),
    )(input)
  }
*/

fn character<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, char, E> {
    let (input, c) = none_of("\"")(input)?;
    if c == '\\' {
        let (input2, c) = anychar(input)?;
        let c2 = match c {
            '"' | '\\' | '/' => c,
            'b' => '\x08',
            'f' => '\x0C',
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            c => c,
        };
        Ok((input2, c2))
    } else {
        Ok((input, c))
    }
}

fn hex_u32<'a, E: ParseError<&'a str> + FromExternalError<&'a str, ParseIntError>>(
    i: &'a str,
) -> IResult<&'a str, Int, E> {
    map_res(
        preceded(
            alt((tag("0x"), tag("0X"))),
            recognize(many1(terminated(
                one_of("0123456789abcdefABCDEF"),
                many0(char('_')),
            ))),
        ),
        |out: &str| Int::from_str_radix(&str::replace(&out, "_", ""), 16),
    )(i)
}

fn param<'a,>(i: &'a str) -> IResult<&'a str, Vec<ParamValue>, VerboseError<&'a str>> {
    context(
        "param",
        preceded(
            char('('),
            cut(terminated(
                separated_list0(preceded(multispace0, char(',')), param_value),
                preceded(multispace0, char(')')),
            )),
        ),
    )(i)
}

type Int = u32;

#[derive(Debug, PartialEq)]
pub enum ParamValue {
    Null,
    Str(String),
    Boolean(bool),
    Num(Int),
}

/// here, we apply the space parser before trying to parse a value
fn param_value<'a, E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError>>(
    i: &'a str,
) -> IResult<&'a str, ParamValue, E> {
    preceded(
        multispace0,
        alt((
            map(string, |s| ParamValue::Str(String::from(s))),
            map(hex_u32, ParamValue::Num),
            map(dec_u32, ParamValue::Num),
            map(boolean, ParamValue::Boolean),
            map(null, |_| ParamValue::Null),
        )),
    )(i)
}

fn identifier<'a>(input: &'a str) -> IResult<&'a str, &'a str, VerboseError<&'a str>> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0_count(alt((alphanumeric1, tag("_")))),
    ))(input)
}

fn func<'a>(input: &'a str) -> IResult<&'a str, (&'a str, Vec<ParamValue>), VerboseError<&'a str>> {
    preceded(multispace1, pair(identifier, preceded(multispace0, param)))(input)
}

/*
fn ws<'a, O, E: ParseError<&'a str>, F: Parser<&'a str, O, E>>(f: F) -> impl Parser<&'a str, O, E> {
    delimited(multispace0, f, multispace0)
}
*/

pub fn command_parse<'a>(i: &'a str) -> IResult<&'a str, Vec<(&'a str, Vec<ParamValue>)>, VerboseError<&'a str>> {
    context("command", preceded(anychar, many1(func)))(i)
}

#[test]
fn main() {
    let i = "c  strlen ( \"hello, \\\" world\", 0x123,0x1234,1234, 456, )   w ()";

    let (rem, ex) = match command_parse(i) {
        Ok(t) => t,
        Err(e) => {
            println!("{i} - {e}");
            std::process::exit(1)
        }
    };

    println!("{i} rem:\"{rem}\" -> {ex:?}");
}
