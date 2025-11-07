use std::collections::HashMap;

pub struct Context<'a> {
    pub si: i32,
    pub env: HashMap<String, i32>,
    pub define_env: &'a HashMap<String, i64>,
    pub define_ptrs: &'a HashMap<String, i64>,
    pub curr_break: u64,
}

impl<'a> Context<'a> {
    pub fn new(define_env: &'a HashMap<String, i64>, define_ptrs: &'a HashMap<String, i64>) -> Self {
        Self { si: 2, env: HashMap::new(), define_env, define_ptrs, curr_break: 0 }
    }
    pub fn with_si(mut self, si: i32) -> Self { self.si = si; self }
}