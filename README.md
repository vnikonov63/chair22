- Use `cargo test -- --test-threads 1` command to run the tests, to avoid data races. Running tests in parallel, especially with the `-c` flag may result in unwanted behaviour of the code.

- Compiler accepts the following arguments:

```
cargo run -- -c tests/test1.snek tests/test1.s
cargo run -- -e tests/test1.snek
cargo run -- -g tests/test1.snek tests/test1.s
cargo run -- -i
```

- The `mode` parameter in `compile_to_instr` function in `main.rs` refers to the destination, where the generated instructions are sent. Conceptually there is a difference between the functionality of each of the destinations. For example, `define` command in boa is only valid in the repl, but would cause a compilation error (not the parser one) if used in JIT execution of the file or the assembly file generation.
