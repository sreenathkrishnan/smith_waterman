
const WORST_CASE_MAX_SCORE : i32 = 0;

#[derive(Copy, Clone)]
enum Moves {
    MATCH,
    INSERT,
    DELETE,
    EMPTY
}

struct OptionScores {
    match_opt  : i32,
    insert_opt : i32,
    delete_opt : i32
}

impl OptionScores {
    fn best_option(&self) -> (i32, Moves) {
        let mut best_move = Moves::EMPTY;
        let mut best_score = WORST_CASE_MAX_SCORE;

        if self.match_opt > best_score {
            best_score = self.match_opt;
            best_move = Moves::MATCH;
        }
        if self.insert_opt > best_score {
            best_score = self.insert_opt;
            best_move = Moves::INSERT;
        }
        if self.delete_opt > best_score {
            best_score = self.delete_opt;
            best_move = Moves::DELETE;
        }
        (best_score, best_move)
    }
}

struct Alignment {
    score_matrix : Vec<Vec<i32> >,
    moves_matrix : Vec<Vec<Moves> >,
    start_position : (i32, i32),
    end_position   : (i32, i32),
    score : i32,
    moves : Vec<Moves>
}

impl Alignment {
    fn new(m : usize, n : usize) -> Alignment { // m and n are the text lengths + 1
        Alignment {
            score_matrix : vec![vec![0; n]; m],
            moves_matrix : vec![vec![Moves::EMPTY; n]; m],
            start_position : (-1,-1),
            end_position   : (-1,-1),
            score : WORST_CASE_MAX_SCORE,
            moves : Vec::new()
        }
    }
    fn backtrace(&mut self) {
        unimplemented!();
    }
}

fn match_score( x: &u8, y: &u8 ) -> i32 {
    if x==y { 2 } else { -1 }
}
fn indel_score( x: &u8) -> i32 {
    match x {
        _ => -1
    }
}

#[allow(non_snake_case)]
pub fn naive_swa( s: &[u8], t: &[u8]) {

    let ns = s.len() + 1; // 1 for blank prefix
    let nt = t.len() + 1;

    let mut align = Alignment::new(ns, nt);

    let mut S = &mut align.score_matrix;
    let mut M = &mut align.moves_matrix;

    for i in 1..ns {
        let x = s[i-1];
        for j in 1..nt {
            let y = t[j-1];
            let opts = OptionScores {
                match_opt  : S[i-1][j-1] + match_score(&x, &y),
                insert_opt : S[i-1][ j ] + indel_score(&x),
                delete_opt : S[ i ][j-1] + indel_score(&y)
            };
            // Currently not tracking parent
            let (score, mov) = opts.best_option(); 
            S[i][j] = score;
            M[i][j] = mov;
            if score > align.score {
                align.score = score;
                align.end_position = ( (i-1) as i32, (j-1) as i32);
            }
        }
    }
    println!(" Best score = {} at end position ({},{})", align.score, align.end_position.0, align.end_position.1);
    println!("{:?}", S);
}