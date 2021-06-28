use std::io::{self, BufRead};

pub fn run() -> io::Result<()> {
    let mut config = String::new();
    let mut sessions = String::new();
    let mut config_done = false;
    for line in io::stdin().lock().lines() {
        let raw_line = line.unwrap();
        if raw_line == "" {
            config_done = true;
        } else {
            if !config_done {
                config = format!("{}\n{}", config, raw_line);
            } else {
                sessions = format!("{}{}", sessions, raw_line);
            }
        }
    }
    println!("{}", config);
    println!("{}", sessions);
    Ok(())
}
