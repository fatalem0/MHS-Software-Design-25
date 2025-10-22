Project of a simple CLI on Rust

Build instructions:
- install Rust toolchain (https://rustup.rs/)
- In the root of the project execute `cargo build -r`
- Run the executable by `./target/release/cli-shell`

Usage examples:
```
>echo "Hello, world!"
Hello, world!
```

```
> echo Some example text > example.txt
> FILE=example.txt
Set FILE=example.txt
> wc < example.txt
       1        3       18
> cat $FILE
Some example text
> rm example.txt
```

```
> x=ex
Set x=ex
> y=it
Set y=it
> $x$y
Goodbye!
```