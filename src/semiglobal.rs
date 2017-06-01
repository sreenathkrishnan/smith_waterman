
use std::i32;
use std::cmp::max;

// Bunch of constants
const NEGATIVE_INF : i32 = i32::MIN / 2; // Dividing by 2 to stay away from the overflow region

// ************* Scoring scheme ************** //
pub struct Scoring {
    pub gap_inititation_score : i32,
    pub gap_unit_score        : i32,
    pub match_score           : i32,
    pub mismatch_score        : i32,
    pub soft_clipping_score   : i32,
}

// ************* Allowed Moves ************** //
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Copy, Clone)]
#[allow(non_camel_case_types)]
pub enum Moves {
    MATCH,
    SUBS,
    INSERT,
    DELETE,
    PREFIX_CLIP,
    SUFFIX_CLIP,
    NONE
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Copy, Clone)]
pub struct Cell {
    pub score : i32,
    pub mov   : Moves,
}

// We are tying to align the read string "t" with the reference "s".
// The read string "t" needs to be consumed completely and hence this
// is a semi-global alignment. 
//
// An affine gap score model is used so that the gap score for a length 'k' is:
// GapScore(k) = gap_inititation_score + gap_unit_score * k 
//
// score_matrix (i,j) is the best semiglobal alignment for prefixes s[0..i], t[0..j] 
//
// match_matrix(i,j) is the best score such that s[i] and t[j] ends in a match
//              .... A   G  s_i
//              .... C   G  t_j
//
// insert_matrix(i,j) is the best score such that s[i] is aligned with a gap
//              .... A   G  s_i
//              .... G  t_j  - 
// This is interpreted as an insertion into "t" w.r.t reference "s" and hence
// called the insert_matrix
//
// delete_matrix(i,j) is the best score such that t[j] is aligned with a gap
//              .... A  s_i  -
//              .... G   G  t_j 
// This is interpreted as a deletion from "t" w.r.t reference "s" and hence
// called the delete_matrix
//
// A soft clipping mode is also implemented wherein you pay a fixed penalty
// to skip a portion at the beginning or end of the read "t". At any point
// in the DP matrix, you can start from the PREFIX_CLIP mode and at any point
// different from the PREFIX_CLIP move you can jump to j=n entry in the
// matrix by a SUFFIX_CLIP move

pub struct SemiglobalAlign {
    // DP Matrices, could be made more memory efficient
    pub score_matrix  : Vec<Vec<Cell> >,
    pub match_matrix  : Vec<Vec<Cell> >,
    pub insert_matrix : Vec<Vec<Cell> >,
    pub delete_matrix : Vec<Vec<Cell> >,

    // For suffix clipping, we need to keep track of best
    // clipping length for each position in reference s
    pub clip_lengths : Vec<usize>,

    // Alignment Outputs
    pub score   : i32,
    pub s_range : [i32; 2],
    pub t_range : [i32; 2],
    pub moves   : Vec<Moves>,
    
    // Clipping specific outputs
    pub prefix_clip_length : usize,
    pub suffix_clip_length : usize
}

impl SemiglobalAlign {
    fn new(m : usize, n : usize) -> SemiglobalAlign { // m and n are the text lengths + 1
        SemiglobalAlign {
            score_matrix  : vec![vec![Cell{score: NEGATIVE_INF, mov: Moves::NONE}; n]; m],
            match_matrix  : vec![vec![Cell{score: NEGATIVE_INF, mov: Moves::NONE}; n]; m],
            insert_matrix : vec![vec![Cell{score: NEGATIVE_INF, mov: Moves::NONE}; n]; m],
            delete_matrix : vec![vec![Cell{score: NEGATIVE_INF, mov: Moves::NONE}; n]; m],

            clip_lengths : vec![0 as usize; m],

            score   : NEGATIVE_INF,
            s_range : [-1, -1], // 2nd index is exclusive
            t_range : [-1, -1], // 2nd index is exclusive
            moves   : Vec::new(),

            prefix_clip_length : 0,
            suffix_clip_length : 0
        }
    }
    #[allow(non_snake_case)]
    pub fn compute( s: &[u8], t: &[u8], scoring: &Scoring ) -> SemiglobalAlign {

        let m = s.len() + 1; // 1 for blank prefix
        let n = t.len() + 1;

        let mut align = SemiglobalAlign::new(m, n);

        {
            let mut S = &mut align.score_matrix;
            let mut M = &mut align.match_matrix;
            let mut I = &mut align.insert_matrix;
            let mut D = &mut align.delete_matrix;
            let mut c = &mut align.clip_lengths;

            // Inititalize the matrices
            M[0][0].score = 0; 
            M[0][0].mov = Moves::NONE;

            I[0][0].score = NEGATIVE_INF; 
            I[0][0].mov = Moves::NONE;

            D[0][0].score = NEGATIVE_INF; 
            D[0][0].mov = Moves::NONE; 

            S[0][0].score = 0; 
            S[0][0].mov = Moves::NONE;

            for i in 1..m {
                M[i][0].score = 0; 
                M[i][0].mov = Moves::NONE;
                I[i][0].score = scoring.gap_inititation_score + scoring.gap_unit_score; // Could start alignment anywhere
                I[i][0].mov = Moves::NONE;
                D[i][0].score = NEGATIVE_INF;
                D[i][0].mov = Moves::NONE;
                S[i][0].score = 0;
                S[i][0].mov = Moves::NONE;
            }

            for j in 1..n {
                M[0][j].score = NEGATIVE_INF;
                M[0][j].mov = Moves::NONE;
                I[0][j].score = NEGATIVE_INF;
                I[0][j].mov = Moves::NONE;
                D[0][j].score = scoring.gap_inititation_score + scoring.gap_unit_score * (j as i32);
                D[0][j].mov = if j==1 { Moves::NONE } else { Moves::DELETE };
                S[0][j] = max( Cell { score: D[0][j].score, mov: Moves::DELETE },
                    Cell { score: scoring.soft_clipping_score, mov: Moves::PREFIX_CLIP } );
            }

            // Core alignment computation
            for i in 1..m {
                let x = s[i-1];
                for j in 1..n {
                    let y = t[j-1];
                    I[i][j] = max ( Cell { score: I[i-1][j].score + scoring.gap_unit_score, mov: Moves::INSERT}, // Already in the insert mode - no initiation
                        Cell { score: S[i-1][j].score + scoring.gap_inititation_score + scoring.gap_unit_score, mov: S[i-1][j].mov}); // Or in some other mode
                    
                    D[i][j] = max ( Cell { score: D[i][j-1].score + scoring.gap_unit_score, mov: Moves::DELETE}, // Already in the delete mode - no initiation
                        Cell { score: S[i][j-1].score + scoring.gap_inititation_score + scoring.gap_unit_score, mov: S[i][j-1].mov }); // Or in some other mode

                    M[i][j] = if x==y {
                        Cell { score: S[i-1][j-1].score + scoring.match_score, mov:S[i-1][j-1].mov }
                    } else {
                        Cell { score: S[i-1][j-1].score + scoring.mismatch_score, mov:S[i-1][j-1].mov }
                    };

                    if j==(n-1) {
                        let temp_max = max ( max ( Cell { score: I[i][j].score, mov: Moves::INSERT }, Cell { score: D[i][j].score, mov: Moves::DELETE }),
                            max ( Cell { score: M[i][j].score, mov: if x==y { Moves::MATCH } else { Moves::SUBS } }, Cell { score: scoring.soft_clipping_score, mov: Moves::PREFIX_CLIP } ) );
                        if temp_max.score > S[i][n-1].score {
                            c[i] = 0;
                            S[i][n-1] = temp_max;
                        }
                    } else {
                        S[i][j] = max ( max ( Cell { score: I[i][j].score, mov: Moves::INSERT }, Cell { score: D[i][j].score, mov: Moves::DELETE }),
                            max ( Cell { score: M[i][j].score, mov: if x==y { Moves::MATCH } else { Moves::SUBS } }, Cell { score: scoring.soft_clipping_score, mov: Moves::PREFIX_CLIP } ) );

                        // Track the score if we do a SUFFIX_CLIP after this character
                        if (S[i][j].score + scoring.soft_clipping_score) > S[i][n-1].score {
                            c[i] = n - 1 - j;
                            S[i][n-1] = Cell { score: S[i][j].score + scoring.soft_clipping_score, mov: Moves::SUFFIX_CLIP };
                        }
                    }

                }
            }

            // It's traceback time
            // In the semiglobal alignment setting, we should scan along j=n and pick the best score
            // Then traceback the moves until j > 0
            align.t_range = [0, (n-1) as i32];

            for i in 0..m {
                if S[i][n-1].score > align.score {
                    align.score = S[i][n-1].score;
                    align.s_range[1] = i as i32;
                }
            }

            align.suffix_clip_length = c[align.s_range[1] as usize];

            let mut i : usize = align.s_range[1] as usize;
            let mut j : usize = (n-1) as usize;
            let mut P = &mut align.moves;

            // This code assumes NON-EMPTY s and t
            P.push( S[i][j].mov );
            let mut last = S[i][j].mov;
            loop {

                let next : Moves;
                match last {
                    Moves::MATCH => { next = M[i][j].mov; i-=1; j-=1; },
                    Moves::SUBS  => { next = M[i][j].mov; i-=1; j-=1; },
                    Moves::INSERT => { next = I[i][j].mov; i-=1; },
                    Moves::DELETE => { next = D[i][j].mov; j-=1; },
                    Moves::PREFIX_CLIP => { align.prefix_clip_length = j; break;},
                    Moves::SUFFIX_CLIP => { j-=align.suffix_clip_length; next = S[i][j].mov;}
                    Moves::NONE => break
                };

                if next == Moves::NONE {
                    break
                }

                P.push(next);

                last = next;
                
            }

            // Compute the start of s_range
            align.s_range[0] = max(align.s_range[1], 0);
            for k in 0..P.len() {
                match P[k] {
                    Moves::MATCH => align.s_range[0]-=1,
                    Moves::SUBS  => align.s_range[0]-=1,
                    Moves::INSERT => align.s_range[0]-=1,
                    Moves::DELETE => {},
                    Moves::PREFIX_CLIP => {},
                    Moves::SUFFIX_CLIP => {},
                    Moves::NONE => panic!("P cannot be NONE. There is a terrible mistake.")
                }
            }
            P.reverse();

        }

        align
    }

    pub fn pretty_print(&self, s: &[u8], t: &[u8]) {

        println!(" Best score = {} ", self.score);
        println!(" s_range = [{},{})", self.s_range[0], self.s_range[1]);
        println!(" t_range = [{},{})", self.t_range[0], self.t_range[1]);
        println!(" Moves : {:?}", self.moves);
        println!(" Prefix clip length : {}", self.prefix_clip_length);
        println!(" Suffix clip length : {}", self.suffix_clip_length);

        let mut line1 = Vec::new();
        let mut line2 = Vec::new();
        let mut line3 = Vec::new();

        let mut i = self.s_range[0] as usize;
        let mut j = self.t_range[0] as usize;
        for m in &self.moves {
            match *m {
                Moves::MATCH => { 
                    line1.push(s[i] as char);
                    line2.push('|');
                    line3.push(t[j] as char);
                    i+=1; j+=1; 
                },
                Moves::SUBS  => { 
                    line1.push(s[i] as char);
                    line2.push('\\');
                    line3.push(t[j] as char);
                    i+=1; j+=1; 
                },
                Moves::INSERT => { 
                    line1.push(s[i] as char);
                    line2.push('+');
                    line3.push('-');
                    i+=1; 
                },
                Moves::DELETE => { 
                    line1.push('-');
                    line2.push('x');
                    line3.push(t[j] as char);
                    j+=1;
                },
                Moves::PREFIX_CLIP => {
                    for k in 0..self.prefix_clip_length {
                        line1.push(' ');
                        line2.push('c');
                        line3.push(t[k] as char);
                    }
                    j = self.prefix_clip_length;
                }
                Moves::SUFFIX_CLIP => {
                    for k in j..t.len() {
                        line1.push(' ');
                        line2.push('c');
                        line3.push(t[k] as char);
                    }
                }
                Moves::NONE => panic!("Moves should not be NONE. This is a terrible mistake! :/")
            }
        }

        for l in line1 {
            print!("{}",l);
        }
        println!("");

        for l in line2 {
            print!("{}",l);
        }
        println!("");

        for l in line3 {
            print!("{}",l);
        }
        println!("");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::Moves::*;
    #[test]
    fn simple_test_semiglobal() {
        let s = b"ACCGTGGATGGG";
        let t = b"GAAAACCGTTGAT";
        let scoring = Scoring {
            gap_inititation_score : -5,
            gap_unit_score : -1,
            match_score : 1,
            mismatch_score : -1,
            soft_clipping_score : -100
        };
        let align = SemiglobalAlign::compute(s, t, &scoring);
        assert_eq!(align.moves, vec![DELETE, DELETE, DELETE, DELETE, MATCH, MATCH, MATCH, MATCH, MATCH, SUBS, MATCH, MATCH, MATCH] );
    }

    #[test]
    fn delete_only_semiglobal() {
        let s = b"TTTT";
        let t = b"AAAA";
        let scoring = Scoring {
            gap_inititation_score : -5,
            gap_unit_score : -1,
            match_score : 1,
            mismatch_score : -3,
            soft_clipping_score : -100
        };
        let align = SemiglobalAlign::compute(s, t, &scoring);
        assert_eq!(align.moves, vec![DELETE, DELETE, DELETE, DELETE] );
    }

    #[test]
    fn insert_in_between_test_semiglobal() {
        let s = b"GGTAGGG";
        let t = b"GGGGG";
        let scoring = Scoring {
            gap_inititation_score : -5,
            gap_unit_score : -1,
            match_score : 1,
            mismatch_score : -3,
            soft_clipping_score : -100
        };
        let align = SemiglobalAlign::compute(s, t, &scoring);
        assert_eq!(align.moves, vec![MATCH, MATCH, INSERT, INSERT, MATCH, MATCH, MATCH] );
    }

    #[test]
    fn prefix_clip_test() {
        let s = b"ATAG";
        let t = b"GGGGGGATG";
        let scoring = Scoring {
            gap_inititation_score : -5,
            gap_unit_score : -1,
            match_score : 1,
            mismatch_score : -1,
            soft_clipping_score : -5
        };
        let align = SemiglobalAlign::compute(s, t, &scoring);
        assert_eq!(align.moves, vec![PREFIX_CLIP, MATCH, MATCH, SUBS]);
    }

    #[test]
    fn suffix_clip_test() {
        let s = b"CGTTTT";
        let t = b"GAAAA";
        let scoring = Scoring {
            gap_inititation_score : -5,
            gap_unit_score : -1,
            match_score : 2,
            mismatch_score : -2,
            soft_clipping_score : -5
        };
        let align = SemiglobalAlign::compute(s, t, &scoring);
        assert_eq!(align.moves, vec![MATCH, SUFFIX_CLIP]);
    }

    #[test]
    fn test_longer_string_all_operations() {
        let s = b"GGGGGGATTTCCCCCCCCCTTTTTTTTTTAAAAAAAAA";
        let t = b"TTTTTGGGGGGATGGCCCCCCTTTTTTTTTTGGGAAAAAAAAAGGGGGG";
        let scoring = Scoring {
            gap_inititation_score : -5,
            gap_unit_score : -1,
            match_score : 2,
            mismatch_score : -2,
            soft_clipping_score : -5
        };
        let align = SemiglobalAlign::compute(s, t, &scoring);
        assert_eq!(align.moves, vec![PREFIX_CLIP, MATCH, MATCH, MATCH, MATCH, MATCH, MATCH, MATCH, MATCH, SUBS, SUBS, MATCH, MATCH, MATCH, MATCH, MATCH, MATCH, INSERT, INSERT, INSERT, MATCH, MATCH, MATCH, MATCH, MATCH, MATCH, MATCH, MATCH, MATCH, MATCH, DELETE, DELETE, DELETE, MATCH, MATCH, MATCH, MATCH, MATCH, MATCH, MATCH, MATCH, MATCH, SUFFIX_CLIP]);
    }
}