mod infra;

success_tests! {
    _1_add1: "1",
    _2_sub1: "2",
    _3_plus: "3",
    _4_minus: "4",
    _5_times: "5",
    _6_simple_let: "6",
    _7_double_simple_let: "7",
    _8_shadowing_simple_let: "8",
    _9_execution_in_expression_binding_let: "9",
    _10_later_binding_available_let: "10",
    _11_nested_let: "11",
    _12_triple_nested_let: "12",
}

failure_tests! {
    _13_unbound_id: "Unbound variable identifier x",
    _14_duplicate: "Duplicate binding",
}