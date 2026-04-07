use std::panic;
use std::fmt;

static TEST_COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
static PASS_COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
static FAIL_COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

pub fn test<F>(name: &str, f: F)
where
    F: FnOnce() + panic::UnwindSafe,
{
    TEST_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let result = panic::catch_unwind(f);
    match result {
        Ok(()) => {
            PASS_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            println!("✓ {}", name);
        }
        Err(e) => {
            FAIL_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            println!("✗ {}", name);
            let msg = if let Some(s) = e.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown error".to_string()
            };
            println!("  Error: {}", msg);
        }
    }
}

pub fn assert_eq<T: fmt::Debug + PartialEq>(expected: T, actual: T) {
    if expected != actual {
        panic!("Assertion failed: {:?} != {:?}", expected, actual);
    }
}

pub fn assert_true(condition: bool) {
    if !condition {
        panic!("Assertion failed: expected true");
    }
}

pub fn assert_false(condition: bool) {
    if condition {
        panic!("Assertion failed: expected false");
    }
}

pub fn assert_none<T>(value: &Option<T>) {
    if value.is_some() {
        panic!("Assertion failed: expected None");
    }
}

pub fn assert_some<T>(value: &Option<T>) {
    if value.is_none() {
        panic!("Assertion failed: expected Some");
    }
}

pub fn assert_panics<F>(f: F)
where
    F: FnOnce() + panic::UnwindSafe,
{
    let result = panic::catch_unwind(f);
    if result.is_ok() {
        panic!("Assertion failed: expected panic");
    }
}

pub fn bench<F>(name: &str, iterations: usize, mut f: F)
where
    F: FnMut(),
{
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        f();
    }
    let elapsed = start.elapsed().as_nanos();
    let per_iter = elapsed / iterations as u128;
    println!("{}: {} ns/iter ({} iterations)", name, per_iter, iterations);
}

pub fn report() -> TestReport {
    let tests = TEST_COUNT.load(std::sync::atomic::Ordering::Relaxed);
    let passed = PASS_COUNT.load(std::sync::atomic::Ordering::Relaxed);
    let failed = FAIL_COUNT.load(std::sync::atomic::Ordering::Relaxed);
    TestReport { tests, passed, failed }
}

#[derive(Debug)]
pub struct TestReport {
    pub tests: usize,
    pub passed: usize,
    pub failed: usize,
}

impl TestReport {
    pub fn success(&self) -> bool {
        self.failed == 0
    }
}

pub struct Mock<T: Clone> {
    expectations: Vec<Expectation<T>>,
    calls: usize,
}

struct Expectation<T> {
    times: usize,
    value: T,
}

impl<T: Clone> Mock<T> {
    pub fn new() -> Self {
        Mock { expectations: Vec::new(), calls: 0 }
    }
    
    pub fn expect(&mut self, value: T, times: usize) {
        self.expectations.push(Expectation { times, value });
    }
    
    pub fn verify(&self) -> Result<(), String> {
        if self.calls < self.expectations.len() {
            Err("Not all expectations were called".to_string())
        } else {
            Ok(())
        }
    }
}
