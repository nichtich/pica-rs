//! This module provides functions to parse bibliographic records encoded in
//! PICA+ and parsers used in the cli commands. PICA+ is the internal format
//! used by the OCLC library system.
//!
//! NOTE: The code to parse excaped strings is based on the nom example; see
//! https://git.io/JkoOn.
//!
//! # PICA+ Grammar
//!
//! ```text
//! Record     ::= Field*
//! Field      ::= Tag Occurrence? Subfield* #x1e
//! Tag        ::= [012] [0-9]{2} [A-Z@]
//! Occurrence ::= '/' [0-9]{2,3}
//! Subfield   ::= Code Value
//! Code       ::= [a-zA-Z0-9]
//! Value      ::= [^#x1e#x1f]
//! ```
//!
//! [EBNF]: https://www.w3.org/TR/REC-xml/#sec-notation

use crate::filter::{BooleanOp, ComparisonOp};
use crate::path::parse_path;
use crate::string::parse_string;
use crate::utils::ws;
use crate::Filter;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::combinator::{all_consuming, map};
use nom::multi::many0;
use nom::sequence::{delimited, preceded, terminated, tuple};
use nom::IResult;

pub(crate) fn parse_comparison_expr(i: &str) -> IResult<&str, Filter> {
    map(
        tuple((
            ws(parse_path),
            alt((
                map(ws(tag("==")), |_| ComparisonOp::Eq),
                map(ws(tag("!=")), |_| ComparisonOp::Ne),
                map(ws(tag("=~")), |_| ComparisonOp::Re),
                map(ws(tag("=^")), |_| ComparisonOp::StartsWith),
                map(ws(tag("=$")), |_| ComparisonOp::EndsWith),
            )),
            ws(parse_string),
        )),
        |e| Filter::ComparisonExpr(e.0, e.1, e.2),
    )(i)
}

pub(crate) fn parse_existence_expr(i: &str) -> IResult<&str, Filter> {
    terminated(map(parse_path, Filter::ExistenceExpr), char('?'))(i)
}

pub(crate) fn parse_not_expr(i: &str) -> IResult<&str, Filter> {
    map(
        preceded(
            ws(char('!')),
            alt((parse_existence_expr, parse_grouped_expr, parse_not_expr)),
        ),
        |e| Filter::NotExpr(Box::new(e)),
    )(i)
}

pub(crate) fn parse_grouped_expr(i: &str) -> IResult<&str, Filter> {
    map(
        delimited(
            char('('),
            alt((parse_boolean_expr, parse_comparison_expr)),
            char(')'),
        ),
        |e| Filter::GroupedExpr(Box::new(e)),
    )(i)
}

fn parse_term_expr(i: &str) -> IResult<&str, Filter> {
    alt((
        parse_comparison_expr,
        parse_existence_expr,
        parse_grouped_expr,
        parse_not_expr,
    ))(i)
}

pub(crate) fn parse_boolean_expr(i: &str) -> IResult<&str, Filter> {
    let (i, (first, remainder)) = tuple((
        parse_term_expr,
        many0(tuple((
            alt((
                map(ws(tag("&&")), |_| BooleanOp::And),
                map(ws(tag("||")), |_| BooleanOp::Or),
            )),
            parse_term_expr,
        ))),
    ))(i)?;

    Ok((
        i,
        remainder.into_iter().fold(first, |prev, (op, next)| {
            Filter::BooleanExpr(Box::new(prev), op, Box::new(next))
        }),
    ))
}

pub fn parse_filter(i: &str) -> IResult<&str, Filter> {
    all_consuming(alt((parse_boolean_expr, parse_term_expr)))(i)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Path;

    #[test]
    fn test_parse_comparison_expr() {
        let path = Path::new("003@", None, '0', None);
        let value = "123456789X".to_string();
        let expected = Filter::ComparisonExpr(path, ComparisonOp::Eq, value);
        assert_eq!(
            parse_comparison_expr("003@.0 == '123456789X'"),
            Ok(("", expected))
        );

        let path = Path::new("003@", None, '0', None);
        let value = "123456789X".to_string();
        let expected = Filter::ComparisonExpr(path, ComparisonOp::Ne, value);
        assert_eq!(
            parse_comparison_expr("003@.0 != '123456789X'"),
            Ok(("", expected))
        );

        let path = Path::new("028@", None, 'd', Some(0));
        let value = "abc".to_string();
        let expected = Filter::ComparisonExpr(path, ComparisonOp::Eq, value);
        assert_eq!(
            parse_comparison_expr("028@.d[0] == 'abc'"),
            Ok(("", expected))
        );

        let path = Path::new("002@", None, '0', None);
        let value = "Tp[12]".to_string();
        let expected = Filter::ComparisonExpr(path, ComparisonOp::Re, value);
        assert_eq!(
            parse_comparison_expr("002@.0 =~ 'Tp[12]'"),
            Ok(("", expected))
        );

        let path = Path::new("002@", None, '0', None);
        let value = "Tp".to_string();
        let expected =
            Filter::ComparisonExpr(path, ComparisonOp::StartsWith, value);
        assert_eq!(parse_comparison_expr("002@.0 =^ 'Tp'"), Ok(("", expected)));

        let path = Path::new("002@", None, '0', None);
        let value = "Tp".to_string();
        let expected =
            Filter::ComparisonExpr(path, ComparisonOp::EndsWith, value);
        assert_eq!(parse_comparison_expr("002@.0 =$ 'Tp'"), Ok(("", expected)));
    }

    #[test]
    fn test_parse_boolean_expr() {
        let term1 = Filter::ComparisonExpr(
            Path::new("003@", None, '0', None),
            ComparisonOp::Eq,
            "123456789X".to_string(),
        );

        let path = Path::new("002@", None, '0', None);
        let value = "Tp1".to_string();
        let term2 = Filter::ComparisonExpr(path, ComparisonOp::Eq, value);

        let path = Path::new("012A", None, '0', None);
        let value = "foo".to_string();
        let term3 = Filter::ComparisonExpr(path, ComparisonOp::Ne, value);

        let expected = Filter::BooleanExpr(
            Box::new(Filter::BooleanExpr(
                Box::new(term1),
                BooleanOp::Or,
                Box::new(term2),
            )),
            BooleanOp::And,
            Box::new(term3),
        );
        assert_eq!(
            parse_boolean_expr(
                "003@.0 == '123456789X' || 002@.0 == 'Tp1' && 012A.0 != 'foo'"
            ),
            Ok(("", expected))
        );

        let path = Path::new("003@", None, '0', None);
        let value = "123456789X".to_string();
        let term1 = Filter::ComparisonExpr(path, ComparisonOp::Eq, value);

        let path = Path::new("002@", None, '0', None);
        let value = "Tp1".to_string();
        let term2 = Filter::ComparisonExpr(path, ComparisonOp::Eq, value);

        let path = Path::new("012A", None, '0', None);
        let value = "foo".to_string();
        let term3 = Filter::ComparisonExpr(path, ComparisonOp::Ne, value);

        let expected = Filter::BooleanExpr(
            Box::new(Filter::GroupedExpr(Box::new(Filter::BooleanExpr(
                Box::new(term1),
                BooleanOp::Or,
                Box::new(term2),
            )))),
            BooleanOp::And,
            Box::new(term3),
        );
        assert_eq!(
            parse_boolean_expr(
                "(003@.0 == '123456789X' || 002@.0 == 'Tp1') && 012A.0 != 'foo'"
            ),
            Ok(("", expected))
        );

        let path = Path::new("003@", None, '0', None);
        let value = "123456789X".to_string();
        let term1 = Filter::ComparisonExpr(path, ComparisonOp::Eq, value);

        let path = Path::new("002@", None, '0', None);
        let value = "Tp1".to_string();
        let term2 = Filter::ComparisonExpr(path, ComparisonOp::Eq, value);

        let path = Path::new("012A", None, '0', None);
        let value = "foo".to_string();
        let term3 = Filter::ComparisonExpr(path, ComparisonOp::Ne, value);

        let expected = Filter::BooleanExpr(
            Box::new(term1),
            BooleanOp::Or,
            Box::new(Filter::GroupedExpr(Box::new(Filter::BooleanExpr(
                Box::new(term2),
                BooleanOp::And,
                Box::new(term3),
            )))),
        );
        assert_eq!(
            parse_boolean_expr(
                "003@.0 == '123456789X' || (002@.0 == 'Tp1' && 012A.0 != 'foo')"
            ),
            Ok(("", expected))
        );
    }

    fn parse_helper(i: &str) -> IResult<&str, String> {
        parse_string(i)
    }

    #[test]
    fn test_parse_string() {
        assert_eq!(parse_helper("'abc'"), Ok(("", String::from("abc"))));
        assert_eq!(parse_helper("'a\tb'"), Ok(("", String::from("a\tb"))));
        assert_eq!(parse_helper("'\u{1f}'"), Ok(("", String::from("\u{1f}"))));
    }
}
