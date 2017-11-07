use combine::*;
use combine::char::space;
use combine::combinator::parser;
use combine::primitives::Consumed;

fn char_in_quotes<I>(input: I) -> ParseResult<char, I>
    where I: Stream<Item = char>,
          I::Error: ParseError<I::Item, I::Range, I::Position> {
    let (c, input) = try!(any().parse_lazy(input).into());
    let mut back_slash_char = satisfy(|c| {
        "\"\\nrtv".chars().any(|x| x == c)
    }).map(|c| {
        match c {
            '"' => '"',
            '\\' => '\\',
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            'v' => '\x0b',
            c => c//Should never happen
        }
    });
    match c {
        '\\' => input.combine(|input| back_slash_char.parse_stream(input)),
        '"' => Err(Consumed::Empty(
            I::Error::empty(input.into_inner().position()).into(),
        )),
        _    => Ok((c, input))
    }
}

parser!{
    fn element[I]()(I) -> String
        where [I: Stream<Item = char>]
    {
        let plain_str =  many1(satisfy(|c| c != ' '));
        let escape_parser = parser(char_in_quotes);
        let quoted_str = between(token('"'), token('"'), many(escape_parser));
        quoted_str.or(plain_str)
    }
}

parser!{
    fn whitespaces[I]()(I) -> ()
    where
        [I: Stream<Item = char>,]
    {
        skip_many1(space())
    }
}

parser!{
    fn tokenize_quoted[I]()(I) -> Vec<String>
    where
        [I: Stream<Item = char>,]
    {
    sep_by1(element(), whitespaces())
    }
}

pub fn tokenize_string(text: &str)-> Option<Vec<String>> {
    let result = tokenize_quoted().easy_parse(State::new(text)).map(|t| t.0);
    result.ok()
}

#[test]
fn tokenize_ok() {
    let text = "foo bar \"foo bar\" \"a test\"";
    let expected = vec!["foo", "bar", "foo bar", "a test"].iter().map(|s| s.to_string()).collect();

    let result = tokenize_quoted().easy_parse(State::new(text)).map(|t| t.0);
    assert_eq!(result, Ok(expected));
}

