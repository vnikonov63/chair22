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
}

failure_tests! {
    _14_unbound_id: "Unbound variable identifier x",
    _15_duplicate: "Duplicate binding",
}

repl_tests! {
    _20_repl_simple_numbers: ["42", "0", "-17"] => ["42", "0", "-17"],
    _21_repl_add1: ["(add1 15)"] => ["16"],
    _22_repl_sub1: ["(sub1 18)"] => ["17"],
    _23_repl_plus: ["(+ 1 17)"] => ["18"],
    _24_repl_minus: ["(- 25 6)"] => ["19"],
    _25_repl_times: ["(* 4 5)"] => ["20"],
    _26_repl_let: ["(let ((x 1) (y 2)) (+ x y))"] => ["3"],
    _27_reple_simple_define: ["(define x 1)"] => [""],
    _28_reple_access_define: ["(define x 1)", "x"] => ["1"],
    _29_repl_expression_define: ["(define x (+ 2 3))", "x"] => ["5"],
    _30_repl_let_in_define: ["(define x (let ((y 1) (z 2)) (+ y z)))", "x"] => ["3"],
    _31_repl_let_shadows_define: ["(define x (let ((x 17) (y 13)) (+ x y)))", "x"] => ["30"],
    _32_repl_use_later1_define: ["(define x 1)", "(+ x 1)"] => ["2"],
    _33_repl_use_later2_define: ["(define x 98)", "(define y 2)", "(+ x y)"] => ["100"],
    _34_repl_use_later_shadows_define: ["(define x 98)", "(define y (let ((x 100) (y 300)) (+ x y)))", "(+ x y)"] => ["498"],
    _35_repl_exit: ["exit"] => ["Thanks for you business with us!"],
    _36_repl_quit: ["quit"] => ["Thanks for you business with us!"],
}