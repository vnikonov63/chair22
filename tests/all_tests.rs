mod infra;

success_tests! {
    simple_let: "10",
}

//TODO: I was able to run tests with the command cargo test -- "cargo run --target x86_64-apple-darwin" 
// But they run twice for some reason, and the test that I wont to run is filtered out
// and I am not using the -c flag as is specified in the write-up

/* failure_tests! {
    unbound_id: "Unbound variable identifier x",
} */