use nt::*;

fn main() {
    let nt = NetworkTables::bind_both("0.0.0.0:1735", "0.0.0.0:1835", "nt-rs");
    loop {}
}