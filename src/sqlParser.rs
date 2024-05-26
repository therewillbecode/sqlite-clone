//! This is a Brainfuck parser and interpreter
//! Run it with the following command:
//! cargo run --example brainfuck -- examples/sample.bf

use chumsky::prelude::*;
use std::{
    env, fs,
    io::{self, Read},
};

#[derive(Clone)]
enum Instr {
    Invalid,
    Left,
    Right,
    Incr,
    Decr,
    Read,
    Write,
    Loop(Vec<Self>),
}

fn parser<'a>() -> impl Parser<'a, &'a str, Vec<Instr>, extra::Err<Simple<'a, char>>> {
    use Instr::*;
    recursive(|bf| {
        choice((
            just('<').to(Left),
            just('>').to(Right),
            just('+').to(Incr),
            just('-').to(Decr),
            just(',').to(Read),
            just('.').to(Write),
        ))
        .or(bf.delimited_by(just('['), just(']')).map(Loop))
        .recover_with(via_parser(nested_delimiters('[', ']', [], |_| Invalid)))
        // .recover_with(skip_then_retry_until([']']))
        .repeated()
        .collect()
    })
}
