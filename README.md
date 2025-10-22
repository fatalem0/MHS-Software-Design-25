Project of a simple CLI on Rust

## Supported functionality:
- can run own implementations of `wc`, `echo`, `cat`, `pwd`
- can run other commands if there is no own implementation
- `exit`, `help`
- setting environment variables
- redirecting `stdin`, `stdout`, `stderr`
- substition of environment variables in weak quotes and in cases without qoutes

## Build and run instructions:
- install Rust toolchain (https://rustup.rs/)
- In the root of the project execute `cargo build -r`
- Run the executable by `./target/release/cli-shell`

## Usage examples:
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