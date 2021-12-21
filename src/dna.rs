use serde::Deserialize;
use std::cmp::PartialEq;
use std::fmt::Debug;
use tokio_pg_mapper_derive::PostgresMapper;

pub fn complement_dna(dna: &str) -> String {
    let mut comp = String::new();
    for b in dna.chars() {
        let bc = match b {
            'A' => 'T',
            'a' => 'T',
            'T' => 'A',
            't' => 'A',
            'C' => 'G',
            'c' => 'G',
            'G' => 'C',
            'g' => 'G',
            _ => 'N',
        };
        comp.push(bc);
    }
    comp
}

fn reverse(s: String) -> String {
    s.chars().rev().collect::<String>()
}

pub fn reverse_complement(dna: &str) -> String {
    reverse(complement_dna(dna))
}

#[derive(Debug, Deserialize, PartialEq, PostgresMapper)]
#[pg_mapper(table = "restriction_enzyme")]
pub struct RestrictionEnzyme {
    pub name: String,
    pub recognition_sequence: String,
}

pub fn find_restriction_sites<'a>(
    dna: &str,
    enzymes: &'a [RestrictionEnzyme],
) -> Vec<(usize, &'a RestrictionEnzyme)> {
    let dna_len = dna.len();
    let mut ret = Vec::new();

    for enzyme in enzymes {
        let rs = &enzyme.recognition_sequence;
        let forward_sites = dna.match_indices(rs);
        for (i, _) in forward_sites {
            ret.push((i + 1, enzyme));
        }

        let rc_rs = reverse_complement(rs);
        let reverse_sites = dna.match_indices(&rc_rs);
        for (i, s) in reverse_sites {
            ret.push((dna_len - s.len() - i + 1, enzyme));
        }
    }

    ret
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reverse_complement_test() {
        assert_eq!(reverse_complement(&"CATAGGTTG".to_string()), "CAACCTATG");
    }

    #[test]
    fn find_restriction_sites_test() {
        let rs = RestrictionEnzyme {
            name: String::from("BamHI"),
            recognition_sequence: String::from("GGATCC"),
        };
        let enzymes = vec![rs];
        let sites = find_restriction_sites("AAAAGGATCC", &enzymes);
        let indexes = sites.iter().map(|&t| t.0).collect::<Vec<_>>();
        assert_eq!(indexes, vec![5, 1]);
    }
}
