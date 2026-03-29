use std::hash::Hash;
use std::collections::HashSet;

pub fn jaccard_index<T>(a: &HashSet<T>, b: &HashSet<T>) -> f64 where T: Eq, T: Hash {
    let union = a.union(b).count() as f64;

    if union == 0.0 {
        return 1.0;
    }

    let intersection = a.intersection(b).count() as f64;

    intersection / union
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jaccard_identical() {
        let a: HashSet<&str> = ["te", "es", "st"].into_iter().collect();
        let b: HashSet<&str> = a.clone();
        assert_eq!(jaccard_index(&a, &b), 1.0);
    }

    #[test]
    fn jaccard_different() {
        let a: HashSet<&str> = ["ab", "bc"].into_iter().collect();
        let b: HashSet<&str> = ["xy", "yz"].into_iter().collect();
        assert_eq!(jaccard_index(&a, &b), 0.0);
    }

    #[test]
    fn jaccard_partial_overlap() {
        let a: HashSet<&str> = ["te", "es", "st"].into_iter().collect();
        let b: HashSet<&str> = ["te", "en", "nt"].into_iter().collect();

        let similarity = jaccard_index(&a, &b);
        assert!(similarity > 0.0 && similarity < 1.0);
    }
}
