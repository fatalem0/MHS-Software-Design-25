use cli_rust::{Init, Repl};

fn main() {
    let mut init = Init::new();
    let mut repl = Repl::new(&init);
    repl.run(&mut init);
}
