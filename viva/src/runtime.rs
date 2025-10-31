#[export_name = "\x01snek_print"]
pub fn snek_print(val: i64) {
    if val == 3 { println!("true"); }
    else if val == 1 { println!("false"); }
    else if val % 2 == 0 { println!("{}", val >> 1); }
    else { println!("Unknown value: {}", val); }
}

#[export_name = "\x01snek_error"]
pub fn snek_error(err_code: i8) { 
    match err_code {
        1 => {
            eprintln!("Runtime error: overflow");
            std::process::exit(1);
        },
        2 => {
            eprintln!("Runtime error: invalid argument");
            std::process::exit(2);
        }
        _ => {
            eprintln!("snek_error called with code = {}", err_code);
        }
    }
}