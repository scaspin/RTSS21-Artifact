fn main() {
    let mut a = 0;
    transaction {
        if true {
            a += 7;
        }
        else {
            a -= 7;
        }
    }
}
