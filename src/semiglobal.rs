
use std::i32;
use std::cmp::max;

// Bunch of constants
const NEGATIVE_INF : i32 = i32::MIN / 2; // Dividing by 2 to stay away from the overflow region

// ************* Scoring scheme ************** //
pub struct Scoring {
    pub gap_inititation_score : i32,
    pub gap_unit_score       : i32,
    pub match_score          : i32,
    pub mismatch_score       : i32,
}

// ************* Allowed Moves ************** //
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Copy, Clone)]
enum Moves {
    MATCH,
    SUBS,
    INSERT,
    DELETE,
    NONE
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

pub struct SemiglobalAlign {
    // DP Matrices, could be made more memory efficient by just tracking previous row only
    score_matrix  : Vec<Vec<i32> >,
    match_matrix  : Vec<Vec<i32> >,
    insert_matrix : Vec<Vec<i32> >,
    delete_matrix : Vec<Vec<i32> >,

    // This matrix need to consructed completely to allow for traceback unless we plan to
    // use a fancier algorithm like Hirshberg's
    moves_matrix  : Vec<Vec<Moves> >,

    // Alignment Outputs
    score   : i32,
    s_range : [i32; 2],
    t_range : [i32; 2],
    moves   : Vec<Moves>
}

impl SemiglobalAlign {
    fn new(m : usize, n : usize) -> SemiglobalAlign { // m and n are the text lengths + 1
        SemiglobalAlign {
            score_matrix  : vec![vec![0; n]; m],
            match_matrix  : vec![vec![0; n]; m],
            insert_matrix : vec![vec![0; n]; m],
            delete_matrix : vec![vec![0; n]; m],
            moves_matrix  : vec![vec![Moves::NONE; n]; m],

            score   : NEGATIVE_INF,
            s_range : [-1, -1], // 2nd index is exclusive
            t_range : [-1, -1], // 2nd index is exclusive
            moves   : Vec::new()
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
            let mut P = &mut align.moves_matrix; // P for Parent, M is taken

            // Inititalize the matrices
            M[0][0] = 0;
            I[0][0] = NEGATIVE_INF;
            D[0][0] = NEGATIVE_INF;
            S[0][0] = max ( max ( M[0][0] , I[0][0] ), D[0][0] );
            P[0][0] = Moves::NONE;

            for i in 1..m {
                M[i][0] = NEGATIVE_INF;
                I[i][0] = scoring.gap_inititation_score + scoring.gap_unit_score * (i as i32);
                D[i][0] = NEGATIVE_INF;
                S[i][0] = I[i][0];
                P[i][0] = Moves::INSERT;
            }
            for j in 1..n {
                M[0][j] = NEGATIVE_INF;
                I[0][j] = NEGATIVE_INF;
                D[0][j] = scoring.gap_inititation_score + scoring.gap_unit_score * (j as i32);
                S[0][j] = D[0][j];
                P[0][j] = Moves::DELETE;
            }

            // Core alignment computation
            for i in 1..m {
                let x = s[i-1];
                for j in 1..n {
                    let y = t[j-1];
                    let insert_opt = ( max ( I[i-1][j],                               // Already in the insert mode - no initiation
                                             S[i-1][j] + scoring.gap_inititation_score // Or in some other mode
                                             ) + scoring.gap_unit_score,              // Unit score need to be added irrespective
                                       Moves::INSERT);

                    let delete_opt = ( max ( D[i][j-1],                               // Already in the delete mode - no initiation
                                             S[i][j-1] + scoring.gap_inititation_score // Or in some other mode
                                             ) + scoring.gap_unit_score,              // Unit score need to be added irrespective
                                       Moves::DELETE);

                    let match_opt = if x==y { 
                                        (S[i-1][j-1] + scoring.match_score, Moves::MATCH) 
                                    } else { 
                                        (S[i-1][j-1] + scoring.mismatch_score, Moves::SUBS)
                                    };
                    let best_opt = max( max( insert_opt, delete_opt ), match_opt ); // There is implicit tie-breaking in this logic

                    I[i][j] = insert_opt.0;
                    D[i][j] = delete_opt.0;
                    M[i][j] = match_opt.0;
                    S[i][j] = best_opt.0;
                    P[i][j] = best_opt.1;

                }
            }
        }

        // It's traceback time
        // In the semiglobal alignment setting, we should scan along j=n and pick the best score
        // Then traceback the moves until j > 0
        align.t_range = [0, (n-1) as i32];

        for i in 0..m {
            if align.score_matrix[i][n-1] > align.score {
                align.score = align.score_matrix[i][n-1];
                align.s_range[1] = i as i32;
            }
        }

        let mut i : usize = align.s_range[1] as usize;
        let mut j : usize = (n-1) as usize ;

        while j > 0 {
            align.moves.push(align.moves_matrix[i][j]);
            match align.moves_matrix[i][j] {
                Moves::MATCH => { i-=1; j-=1; },
                Moves::SUBS  => { i-=1; j-=1; },
                Moves::INSERT => { i-=1; },
                Moves::DELETE => { j-=1; },
                Moves::NONE => panic!("Moves should not be NONE. This is a terrible mistake! :/")
            }
        }
        align.s_range[0] = i as i32;
        align.moves.reverse();

        align
    }

    pub fn pretty_print(&self, s: &[u8], t: &[u8]) {

        println!(" Best score = {} ", self.score);
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
            mismatch_score : -1
        };
        let mut align = SemiglobalAlign::compute(s, t, &scoring);
        assert_eq!(align.moves, [DELETE, DELETE, DELETE, DELETE, MATCH, MATCH, MATCH, MATCH, MATCH, SUBS, MATCH, MATCH, MATCH] );
    }
}