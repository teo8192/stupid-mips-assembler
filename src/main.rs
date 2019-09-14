pub mod ma;

use ma::assemble_file;
use std::env;
use std::process;

fn main() {
    let mut args = env::args();
    args.next();
    let filename = if let Some(filename) = args.next() {
        filename
    } else {
        println!("Must have filename as argument");
        process::exit(1);
    };
    assemble_file(filename).unwrap_or_else(|err| {
        eprintln!("problem when assembling file {}", err);
        process::exit(1);
    });
}
