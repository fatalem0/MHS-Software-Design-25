use cli_rust::{Init, Repl};

fn main() {
    let init = Init::new();
    let mut repl = Repl::new(&init);
    repl.run();
}
