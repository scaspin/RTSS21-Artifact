fn foo() -> i32 {
    let mut a = 2;
    transaction {
        if a > 0 {
            return a;
        }
        a += 3;
        return a;
    }
}

fn main() {
    let mut _b = foo();
    _b -= 1;
}
