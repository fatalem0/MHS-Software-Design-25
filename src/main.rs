use cli_rust::{Init, Repl};

fn main() {
    let init = Init::new();
    let repl = Repl::new(&init);
    repl.run();
}
