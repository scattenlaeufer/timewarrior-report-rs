# timewarrior-report-rs

A crate to read the data passed by creating a
[Timewarrior](https://timewarrior.net/) report, written in Rust.

## Usage

This is a basic example to read the data for a Timewarrior report from `stdin` and print it:

```rust
use timewarrior_report::TimewarriorData;

fn main() {
   let report_data = TimewarriorData::from_stdin();
   dbg!(report_data);
}
```
