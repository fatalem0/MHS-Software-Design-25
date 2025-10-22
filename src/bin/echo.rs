use std::env;

/// Echo command implementation as a standalone executable
/// Prints all arguments separated by spaces to stdout
fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        println!();
    } else {
        println!("{}", args.join(" "));
    }
}
