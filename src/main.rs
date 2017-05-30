

extern crate smith_waterman;

use smith_waterman::semiglobal::*;

fn main() {
    let s = b"ACCGTGGATGGG";
    let t = b"GAAAACCGTTGAT";
    let scoring = Scoring {
        gap_inititation_score : -5,
        gap_unit_score : -1,
        match_score : 1,
        mismatch_score : -1
    };
    let mut align = SemiglobalAlign::compute(s, t, &scoring);
    align.pretty_print(s,t);
}
