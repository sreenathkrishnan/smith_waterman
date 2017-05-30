

extern crate smith_waterman;

use smith_waterman::semiglobal::SemiglobalAlign;

fn main() {
    let s = b"ACCGTGGATGGG";
    let t = b"GAAAACCGTTGAT";
    let mut align = SemiglobalAlign::compute(s, t);
    align.pretty_print(s,t);
}
