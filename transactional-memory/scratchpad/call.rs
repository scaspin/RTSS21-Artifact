fn foo() {
    let mut b = 3;
    b -= 1;
    transaction {
        let mut _a = 2;
        _a += 3;
    }
    b += 4;
}

fn main() {
    foo();
    foo();
    foo();
}
