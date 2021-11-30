use std::cmp::min;

#[allow(clippy::needless_range_loop)]
pub fn levenshtein(a: &str, b: &str) -> usize {
    let w1 = a.chars().collect::<Vec<_>>();
    let w2 = b.chars().collect::<Vec<_>>();

    let a_max = a.len() + 1;
    let b_max = b.len() + 1;

    // create distance matrix
    let mut d = vec![vec![0; b_max]; a_max];

    for i in 1..a_max {
        d[i][0] = i;
    }
    for j in 1..b_max {
        d[0][j] = j;
    }

    for i in 1..a_max {
        for j in 1..b_max {
            let cost = if w1[i - 1] == w2[j - 1] {
                d[i - 1][j - 1]
            } else {
                1 + min(min(d[i][j - 1], d[i - 1][j]), d[i - 1][j - 1])
            };
            d[i][j] = cost;
        }
    }

    d[a_max - 1][b_max - 1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levenshtein_test() {
        assert_eq!(levenshtein("CTCGAG", "CTCGAC"), 1);
        assert_eq!(levenshtein("CTCGA", "CTCGAC"), 1);
        assert_eq!(levenshtein("CTCGAG", "CTCGA"), 1);
    }
}
