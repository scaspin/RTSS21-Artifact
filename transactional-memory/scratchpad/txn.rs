fn main() {
    foo();
}

fn foo() -> i32 {
    let mut a = 1;
    transaction {
        a += 1;
    }
    transaction {
        a += 2;
    }
    transaction {
        a += 3;
    }
    return a;
}
