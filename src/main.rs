mod repl;
use repl::repl_loop;

fn main() {
    repl_loop().expect("something went wrong in REPL");
}
