use std::ffi::OsStr;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::os::windows::ffi::OsStrExt;

use std::iter;

fn main() {
    let system_classes_file = File::open("system_classes.ini").unwrap();
    let mut out_file = File::create("src/ffi/system_classes.rs").unwrap();

    let system_classes_file = BufReader::new(system_classes_file);

    for line in system_classes_file.lines().map(Result::unwrap) {
        let eq = line.find('=').expect("INI file is not in the proper format!");

        let ident = &line[..eq];
        let val: &OsStr = line[eq + 1..].as_ref();

        let val_u16: Vec<u16> = val.encode_wide()
            .chain(iter::once(0)).collect();

        writeln!(out_file, "pub const {}: SystemClass = SystemClass(&{:?});", ident, val_u16).unwrap();
    }
}
