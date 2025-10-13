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
    _1_fail_unbound_id: "Unbound variable identifier x",
    _2_fail_duplicate: "Duplicate binding",
    _3_fail_empty: "Invalid: parse error",
    _4_fail_unclosed1: "Invalid: parse error",
    _5_fail_unclosed2: "Invalid: parse error",
    _6_fail_unclosed3: "Invalid: parse error",
    _7_fail_wrong_command1: "Invalid: parse error",
    _8_fail_wrong_command2: "Invalid: parse error",
    _9_fail_define_aot: "Invalid: parse error",
    _10_fail_define_inside: "Invalid: parse error",
}

repl_tests! {
    _1_repl_simple_numbers: ["42", "0", "-17"] => ["42", "0", "-17"],
    _2_repl_add1: ["(add1 15)"] => ["16"],
    _3_repl_sub1: ["(sub1 18)"] => ["17"],
    _4_repl_plus: ["(+ 1 17)"] => ["18"],
    _5_repl_minus: ["(- 25 6)"] => ["19"],
    _6_repl_times: ["(* 4 5)"] => ["20"],
    _7_repl_let: ["(let ((x 1) (y 2)) (+ x y))"] => ["3"],
    _8_repl_simple_define: ["(define x 1)"] => [""],
    _9_repl_access_define: ["(define x 1)", "x"] => ["1"],
    _10_repl_expression_define: ["(define x (+ 2 3))", "x"] => ["5"],
    _11_repl_let_in_define: ["(define x (let ((y 1) (z 2)) (+ y z)))", "x"] => ["3"],
    _12_repl_let_shadows_define: ["(define x (let ((x 17) (y 13)) (+ x y)))", "x"] => ["30"],
    _13_repl_use_later1_define: ["(define x 1)", "(+ x 1)"] => ["2"],
    _14_repl_use_later2_define: ["(define x 98)", "(define y 2)", "(+ x y)"] => ["100"],
    _15_repl_use_later_shadows_define: ["(define x 98)", "(define y (let ((x 100) (y 300)) (+ x y)))", "(+ x y)"] => ["498"],
    _16_parse_error_no_exit: ["(hello", "(+ 1 2)"] => ["Invalid: parse error", "3"],
    _17_dublicate_binding_no_exit: ["(define x 4)", "(define x 3)", "(+ 1 4)"] => ["Duplicate binding", "5"],
    _18_repl_exit: ["exit"] => ["Thanks for you business with us!"],
    _19_repl_quit: ["quit"] => ["Thanks for you business with us!"],
}