fn main() {
	let mut a = 0;
	transaction {
		while a < 4 {
			a += 1;
		}
	}
}
