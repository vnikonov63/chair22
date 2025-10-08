mod infra;

success_tests! {
    simple_let: "10",
}

failure_tests! {
    unbound_id: "Unbound variable identifier x",
}