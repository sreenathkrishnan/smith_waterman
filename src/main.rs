

extern crate smith_waterman;

use smith_waterman::semiglobal::*;

fn main() {
    let s = b"GGAGGG";
    let t = b"GGGGGG";
    let scoring = Scoring {
        gap_inititation_score : -5,
        gap_unit_score : -1,
        match_score : 1,
        mismatch_score : -1
    };
    let align = SemiglobalAlign::compute(s, t, &scoring);
    align.pretty_print(s,t);
    // for row in align.match_matrix {
    //     for e in row {
    //         print!("{:?} ", e.score);
    //     }
    //     println!("");
    // }
    // println!("{:?}",align.moves);
}
