mod repl;
use repl::repl_loop;

mod sql_parser;

fn main() {
    repl_loop().expect("something went wrong in REPL");
}
