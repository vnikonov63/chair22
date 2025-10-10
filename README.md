Use `cargo test -- --test-threads 1` command to run the tests, to avoid data races. Running tests in parallel, especially with the `-c` flag may result in unwanted behaviour of the code.

Compiler accepts the following arguments:

```
cargo run -- -c tests/test1.snek tests/test1.s
cargo run -- -e tests/test1.snek
cargo run -- -g tests/test1.snek tests/test1.s
cargo run -- -i
```
