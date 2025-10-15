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
    _13_complex_let: "13",
    _14_super_complex_let: "14",
}

failure_tests! {
    _fail_1_unbound_id: "Unbound variable identifier x",
    _fail_2_duplicate: "Duplicate binding",
    _fail_3_empty: "Invalid: parse error",
    _fail_4_unclosed1: "Invalid: parse error",
    _fail_5_unclosed2: "Invalid: parse error",
    _fail_6_unclosed3: "Invalid: parse error",
    _fail_7_wrong_command1: "Invalid: parse error",
    _fail_8_wrong_command2: "Invalid: parse error",
    _fail_9_define_aot: "Invalid: parse error",
    _fail_10_define_inside: "Invalid: parse error",
}

repl_tests! {
    _repl_1_simple_numbers: ["42", "0", "-17"] => ["42", "0", "-17"],
    _repl_2_add1: ["(add1 15)"] => ["16"],
    _repl_3_sub1: ["(sub1 18)"] => ["17"],
    _repl_4_plus: ["(+ 1 17)"] => ["18"],
    _repl_5_minus: ["(- 25 6)"] => ["19"],
    _repl_6_times: ["(* 4 5)"] => ["20"],
    _repl_7_let: ["(let ((x 1) (y 2)) (+ x y))"] => ["3"],
    _repl_8_simple_define: ["(define x 1)"] => [""],
    _repl_9_access_define: ["(define x 1)", "x"] => ["1"],
    _repl_10_expression_define: ["(define x (+ 2 3))", "x"] => ["5"],
    _repl_11_let_in_define: ["(define x (let ((y 1) (z 2)) (+ y z)))", "x"] => ["3"],
    _repl_12_let_shadows_define: ["(define x (let ((x 17) (y 13)) (+ x y)))", "x"] => ["30"],
    _repl_13_use_later1_define: ["(define x 1)", "(+ x 1)"] => ["2"],
    _repl_14_use_later2_define: ["(define x 98)", "(define y 2)", "(+ x y)"] => ["100"],
    _repl_15_use_later_shadows_define: ["(define x 98)", "(define y (let ((x 100) (y 300)) (+ x y)))", "(+ x y)"] => ["498"],
    _repl_16_parse_error_no_exit: ["(hello", "(+ 1 2)"] => ["Invalid: parse error", "3"],
    _repl_17_dublicate_binding_no_exit: ["(define x 4)", "(define x 3)", "(+ 1 4)"] => ["Duplicate binding", "5"],
    _repl_18_unbound_variable_identifier: ["y"] => ["Unbound variable identifier y"],
}