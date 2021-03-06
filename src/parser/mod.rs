use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::{cut, map, opt};
use nom::IResult;
use nom::multi::many0;
use nom::sequence::{delimited, terminated};

use crate::ast::*;
use crate::lexer::{at_keyword, parse, symbol, token, junk, ident};
use crate::parser::selector::{id_selector, class_selector, selector_group};
use crate::parser::mixin::{mixin_simple_selector, mixin_selector};
use crate::parser::value::{variable_declaration_value, declaration_value};

#[cfg(test)]
mod tests;

mod value;
mod string;
mod selector;
mod mixin;

fn parse_stylesheet(input: &str) -> IResult<&str, Stylesheet> {
    parse(stylesheet)(input)
}

fn stylesheet(input: &str) -> IResult<&str, Stylesheet> {
    let (input, items) = list_of_items(input)?;
    Ok((input, Stylesheet { items }))
}

fn block_of_items(input: &str) -> IResult<&str, Vec<Item>> {
    delimited(
        symbol("{"),
        cut(list_of_items),
        symbol("}"),
    )(input)
}

fn list_of_items(input: &str) -> IResult<&str, Vec<Item>> {
    many0(item)(input)
}

fn item(input: &str) -> IResult<&str, Item> {
    // FIXME: There is a lot of backtracking going on here
    // TODO: Support regular function calls (specifically each(...) calls)
    alt((
        mixin_declaration,
        declaration,
        mixin_call,
        qualified_rule,
        variable_declaration,
        variable_call,
//        at_rule,
    ))(input)
}

fn declaration(input: &str) -> IResult<&str, Item> {
    // TODO: Parse LESS property merge syntax

    let (input, name) = token(ident)(input)?;
    let (input, _) = symbol(":")(input)?;
    let (input, value) = declaration_value(input)?;
    let (input, important) = important(input)?;
    let (input, _) = symbol(";")(input)?;
    Ok((input, Item::Declaration { name, value, important }))
}

/// Parse an !important token
fn important(input: &str) -> IResult<&str, bool> {
    map(opt(symbol("!important")), |o| o.is_some())(input)
}

fn qualified_rule(input: &str) -> IResult<&str, Item> {
    // TODO: Parse guard

    let (input, selector_group) = selector_group(input)?;
    let (input, block) = block_of_items(input)?;
    Ok((input, Item::QualifiedRule { selector_group, block }))
}

//fn at_rule(input: &str) -> IResult<&str, Item> {
//    let (input, name) = at_keyword(input)?;
//}

fn mixin_declaration(input: &str) -> IResult<&str, Item> {
    // TODO: Parse arguments
    // TODO: Parse guard

    let (input, selector) = token(mixin_simple_selector)(input)?;
    let (input, _) = symbol("()")(input)?;
    let (input, block) = block_of_items(input)?;
    Ok((input, Item::MixinDeclaration { selector, block }))
}

fn mixin_call(input: &str) -> IResult<&str, Item> {
    // TODO: Parse arguments
    // TODO: Parse lookups

    let (input, selector) = mixin_selector(input)?;
    let (input, _) = symbol("()")(input)?;
    let (input, _) = symbol(";")(input)?;
    Ok((input, Item::MixinCall { selector }))
}

fn variable_declaration(input: &str) -> IResult<&str, Item> {
    let (input, name) = at_keyword(input)?;
    let (input, _) = symbol(":")(input)?;
    let (input, value) = variable_declaration_value(input)?;
    let (input, _) = symbol(";")(input)?;
    Ok((input, Item::VariableDeclaration { name, value }))
}

fn variable_call(input: &str) -> IResult<&str, Item> {
    let (input, name) = at_keyword(input)?;
    let (input, _) = symbol("()")(input)?;
    let (input, _) = symbol(";")(input)?;
    Ok((input, Item::VariableCall { name }))
}
