use std::io::{self, BufRead, Read};

fn main() -> io::Result<()> {
    let mut input = String::new();
    for line in io::stdin().lock().lines() {
        println!("{:?}", line.unwrap());
    }
    Ok(())
}
