use crate::{parser::parse_program, sema::analyze};

#[test]
fn allows_custom_decorator() {
    let src = r#"
package main

fn foo(next: func() string) -> func() string {
    return next
}

@foo
fn run() -> string {
    return "ok"
}
"#;
    let mut program = parse_program(src).expect("parse ok");
    let result = analyze(&mut program);
    result.expect("sema should succeed");
}

#[test]
fn enforces_retry_on_error_functions() {
    let src = r#"
package main

@retry(3)
fn run() -> string {
    return "ok"
}
"#;
    let mut program = parse_program(src).expect("parse ok");
    let result = analyze(&mut program);
    assert!(result.is_err());
}

#[test]
fn catches_non_exhaustive_match() {
    let src = r#"
package main

enum Status {
    Pending
    Running
    Done
}

fn label(s: Status) -> string {
    match s {
        Status::Pending => "pending",
        Status::Running => "running",
    }
}
"#;
    let mut program = parse_program(src).expect("parse ok");
    let result = analyze(&mut program);
    assert!(result.is_err());
}

#[test]
fn checks_match_exhaustiveness_inside_for() {
    let src = r#"
package main

enum Status {
    Pending
    Running
    Done
}

fn scan(s: Status) {
    for true {
        match s {
            Status::Pending => {}
            Status::Running => {}
        }
    }
}
"#;
    let mut program = parse_program(src).expect("parse ok");
    // Before structured `for` parsing, the loop body was opaque raw text and the
    // nested non-exhaustive match (missing `Done`) went unchecked. Now the body
    // is a real block, so semantic analysis reaches inside the loop.
    let result = analyze(&mut program);
    assert!(
        result.is_err(),
        "non-exhaustive match nested in a for loop must be caught"
    );
}

#[test]
fn checks_match_exhaustiveness_inside_switch() {
    let src = r#"
package main

enum Status {
    Pending
    Running
    Done
}

fn scan(n: int, s: Status) {
    switch n {
    case 1:
        match s {
            Status::Pending => {}
            Status::Running => {}
        }
    }
}
"#;
    let mut program = parse_program(src).expect("parse ok");
    // The switch case body is now a real block, so the nested non-exhaustive
    // match (missing `Done`) is analyzed instead of being opaque raw text.
    let result = analyze(&mut program);
    assert!(
        result.is_err(),
        "non-exhaustive match nested in a switch case must be caught"
    );
}

#[test]
fn validates_custom_decorator_arity_on_methods() {
    let src = r#"
package main

fn trace(next: func(x int) int, label: string) -> func(x int) int {
    return next
}

struct T {
    v: int
}

impl T {
    @trace
    fn compute(self, x: int) -> int {
        return x
    }
}
"#;
    let mut program = parse_program(src).expect("parse ok");
    // `@trace` expects 1 argument (label) but is applied with 0. Previously this
    // went unchecked on methods (an empty known_functions map was passed).
    let result = analyze(&mut program);
    assert!(
        result.is_err(),
        "wrong-arity custom decorator on a method must be caught"
    );
}

#[test]
fn accepts_correct_custom_decorator_on_method() {
    let src = r#"
package main

fn trace(next: func(x int) int, label: string) -> func(x int) int {
    return next
}

struct T {
    v: int
}

impl T {
    @trace("compute")
    fn compute(self, x: int) -> int {
        return x
    }
}
"#;
    let mut program = parse_program(src).expect("parse ok");
    let result = analyze(&mut program);
    result.expect("a correctly-applied custom decorator on a method should analyze");
}

#[test]
fn rejects_memoize_with_slice_param() {
    let src = r#"
package main

@memoize
fn sum(items: []int) -> int {
    return 1
}
"#;
    let mut program = parse_program(src).expect("parse ok");
    let result = analyze(&mut program);
    assert!(result.is_err());
}

#[test]
fn rejects_memoize_on_methods() {
    let src = r#"
package main

struct User {
    name: string
}

impl User {
    @memoize
    fn greet(self) -> string {
        return "hi"
    }
}
"#;
    let mut program = parse_program(src).expect("parse ok");
    let result = analyze(&mut program);
    assert!(result.is_err());
}

#[test]
fn rejects_duplicate_declarations_and_reserved_generated_names() {
    let src = r#"
package main

fn load() {}
fn load() {}
fn mainWarp() {}
"#;
    let mut program = parse_program(src).expect("parse ok");
    let result = analyze(&mut program);
    let diagnostics = result.expect_err("sema should fail");
    assert!(diagnostics.iter().any(|diag| diag.code == "E0100"));
    assert!(diagnostics.iter().any(|diag| diag.code == "E0101"));
}

#[test]
fn rejects_tagged_match_payload_arity_and_duplicate_arms() {
    let src = r#"
package main

enum Result<T> {
    Ok(T)
    Err(string)
}

fn label(r: Result<int>) -> string {
    match r {
        Ok => "ok",
        Ok(v) => "again",
        Err(e) => e,
    }
}
"#;
    let mut program = parse_program(src).expect("parse ok");
    let result = analyze(&mut program);
    let diagnostics = result.expect_err("sema should fail");
    assert!(diagnostics.iter().any(|diag| diag.code == "E0200"));
    assert!(diagnostics.iter().any(|diag| diag.code == "E0201"));
}

#[test]
fn validates_known_custom_decorator_argument_count() {
    let src = r#"
package main

fn trace(next: func() string, label: string) -> func() string {
    return next
}

@trace
fn run() -> string {
    return "ok"
}
"#;
    let mut program = parse_program(src).expect("parse ok");
    let result = analyze(&mut program);
    let diagnostics = result.expect_err("sema should fail");
    assert!(diagnostics.iter().any(|diag| diag.code == "E0302"));
}
