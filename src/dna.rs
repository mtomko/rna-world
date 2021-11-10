fn complement_dna(dna: String) -> String {
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

pub fn reverse_complement(dna: String) -> String {
    reverse(complement_dna(dna))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reverse_complement_test() {
        assert_eq!(reverse_complement("CATAGGTTG".to_string()), "CAACCTATG");
    }
}
