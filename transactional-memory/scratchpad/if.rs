fn main() {
    let mut a = 0;
    // SLOW: open a file
    transaction {
        if (a < 6) {
            a += 7;
        }
        else {
            a -= 7;
        }
    }
}
