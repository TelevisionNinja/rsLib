use std::cmp::max;
use std::collections::HashMap;
use std::collections::HashSet;
use super::aho_corasick::AhoCorasick;

pub struct AhoCorasickFilter {
    inner: AhoCorasick
}

impl AhoCorasickFilter {
    pub fn new() -> Self {
        Self { inner: AhoCorasick::new() }
    }

    pub fn insert(&mut self, word: &str) {
        self.inner.insert(word)
    }

    pub fn remove(&mut self, word: &str) {
        self.inner.remove(word)
    }

    pub fn search(&self, text: &str) -> Vec<(usize, usize)> {
        self.inner.search(text)
    }

    pub fn build(&mut self, words: Vec<&str>) {
        self.inner.build(words)
    }

    /**
     * time: O(n)
     * returns filtered string
     */
    pub fn filter(&self, string: &str, censored_string: &str) -> String {
        let mut node = self.inner.root;
        let mut indices: HashMap<usize, usize> = HashMap::new();

        // empty string case is removed as it does not make sense in a filtering function

        let characters: Vec<_> = string.chars().collect();
        let mut i = 0;

        while i < characters.len() {
            let c = characters[i];

            if let Some(&next) = self.inner.nodes.get(&node).unwrap().children.get(&c) {
                node = next;
                i += 1;

                for &out_node in &self.inner.nodes.get(&node).unwrap().output_links {
                    let len = self.inner.nodes.get(&out_node).unwrap().length;
                    let start_index = i - len;

                    if let Some(current) = indices.get(&start_index) {
                        indices.insert(start_index, max(*current, len));
                    }
                    else {
                        indices.insert(start_index, len);
                    }
                }
            }
            else if node == self.inner.root {
                i += 1;
            }
            else {
                node = self.inner.nodes.get(&node).unwrap().suffix_link.unwrap();
            }
        }

        //---------

        let mut output: String = String::new();
        let mut j = 0;
        
        while j < string.len() {
            if indices.contains_key(&j) {
                // the found longer bound is always in the array bounds because of the DFA
                // loop is unrolled by 1 iteration to have all loop logic in the required iterations
                // all accept indices will have return lengths greater than 0
                output += censored_string;

                let mut end_index = j + indices.get(&j).unwrap();
                j += 1;

                while j < end_index {
                    /*
                    intersection case:
                    0 * * *
                      1 * * * * *

                    subset case:
                    0 * * * *
                      1 * *

                    disjoint case:
                    0 * * * _ _
                                6 * * *
                    */

                    if let Some(index_value) = indices.get(&j) {
                        let second_end_index = j + index_value;
                        end_index = max(second_end_index, end_index);
                    }

                    output += censored_string;
                    j += 1;
                }
            }
            else {
                output += &characters[j].to_string();
                j += 1;
            }
        }

        output
    }

    /**
     * time: O(n)
     * returns filtered string
     */
    pub fn filter_and_ignore_chars(&self, string: &str, censored_string: &str, ignore_chars: HashSet<char>) -> String {
        let mut node = self.inner.root;
        let mut indices: HashMap<usize, usize> = HashMap::new();

        // empty string case is removed as it does not make sense in a filtering function

        let characters: Vec<_> = string.chars().collect();
        let mut i = 0;

        while i < characters.len() {
            let c = characters[i];

            if ignore_chars.contains(&c) {
                i += 1;
            }
            else if let Some(&next) = self.inner.nodes.get(&node).unwrap().children.get(&c) {
                node = next;

                for &out_node in &self.inner.nodes.get(&node).unwrap().output_links {
                    let len = self.inner.nodes.get(&out_node).unwrap().length;

                    if len != 0 {
                        if let Some(current) = indices.get(&i) {
                            indices.insert(i, max(*current, len));
                        }
                        else {
                            indices.insert(i, len);
                        }
                    }
                }

                i += 1;
            }
            else if node == self.inner.root {
                i += 1;
            }
            else {
                node = self.inner.nodes.get(&node).unwrap().suffix_link.unwrap();
            }
        }

        //---------

        let mut output: String = String::new();
        let mut j = string.len();

        while j > 0 {
            j -= 1;

            if ignore_chars.contains(&characters[j]) || !indices.contains_key(&j) {
                output += &characters[j].to_string();
            }
            else {
                // the found bound is always in the array bounds because of the DFA
                // loop is unrolled by 1 iteration to have all loop logic in the required iterations
                // all accept indices will have return lengths greater than 0

                output += censored_string;

                let mut length = *indices.get(&j).unwrap();
                let mut k = 1;

                while k < length {
                    j -= 1;

                    /*
                    intersection case:
                    0 * * *
                      1 * * * * *

                    subset case:
                    0 * * * *
                      1 * *

                    disjoint case:
                    0 * * * _ _
                                6 * * *
                    */

                    if ignore_chars.contains(&characters[j]) {
                        output += &characters[j].to_string();
                    }
                    else {
                        if let Some(other_length) = indices.get(&j) {
                            if length - k < *other_length {
                                length = other_length + k; // no minus 1 because k is 1 behind j here
                            }
                        }

                        output += censored_string;
                        k += 1;
                    }
                }
            }
        }

        output.chars().rev().collect::<String>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vector_pair_to_string(pairs: Vec<(usize, usize)>) -> String {
        if pairs.is_empty() {
            return "".to_string();
        }

        pairs
            .iter()
            .map(|(index, length)| format!("({} {})", index, length))
            .collect::<Vec<_>>()
            .join(", ")
    }

    #[test]
    fn aho_corasick_tests() {
        let mut aho_corasick_filter = AhoCorasickFilter::new();

        assert_eq!(vector_pair_to_string(aho_corasick_filter.search("")), "");
        assert_eq!(vector_pair_to_string(aho_corasick_filter.search("apple")), "");

        let word_list = vec!["apple", "app", "bat"];
        aho_corasick_filter.build(word_list);

        assert_eq!(vector_pair_to_string(aho_corasick_filter.search("apple")), "(0 3), (0 5)");
        assert_eq!(vector_pair_to_string(aho_corasick_filter.search("app")), "(0 3)");
        assert_eq!(vector_pair_to_string(aho_corasick_filter.search("bat")), "(0 3)");
        assert_eq!(vector_pair_to_string(aho_corasick_filter.search("batapple")), "(0 3), (3 3), (3 5)");

        aho_corasick_filter.remove("apple");
        assert_eq!(vector_pair_to_string(aho_corasick_filter.search("apple")), "(0 3)");
        assert_eq!(vector_pair_to_string(aho_corasick_filter.search("app")), "(0 3)");

        aho_corasick_filter.remove("app");
        assert_eq!(vector_pair_to_string(aho_corasick_filter.search("app")), "");
        assert_eq!(vector_pair_to_string(aho_corasick_filter.search("bat")), "(0 3)");

        aho_corasick_filter.insert("apple");
        aho_corasick_filter.insert("app");
        assert_eq!(vector_pair_to_string(aho_corasick_filter.search("apple")), "(0 3), (0 5)");
        assert_eq!(vector_pair_to_string(aho_corasick_filter.search("app")), "(0 3)");

        assert_eq!(vector_pair_to_string(aho_corasick_filter.search("")), "");
        aho_corasick_filter.remove("bat");
        assert_eq!(vector_pair_to_string(aho_corasick_filter.search("")), "");

        aho_corasick_filter.insert("");
        assert_eq!(vector_pair_to_string(aho_corasick_filter.search("")), "(0 0)");
        aho_corasick_filter.remove("");
        assert_eq!(vector_pair_to_string(aho_corasick_filter.search("")), "");

        aho_corasick_filter.insert("i");
        aho_corasick_filter.insert("in");
        aho_corasick_filter.insert("tin");
        aho_corasick_filter.insert("sting");
        assert_eq!(
            vector_pair_to_string(aho_corasick_filter.search("stings")) == "(2 1), (1 3), (2 2), (0 5)" ||
            vector_pair_to_string(aho_corasick_filter.search("stings")) == "(2 1), (2 2), (1 3), (0 5)",
            true
        );

        //-----

        assert_eq!(aho_corasick_filter.filter("apple", "*"), "*****");
        assert_eq!(aho_corasick_filter.filter("app", "*"), "***");
        assert_eq!(aho_corasick_filter.filter("bat", "*"), "bat");
        assert_eq!(aho_corasick_filter.filter("batapple", "*"), "bat*****");
        assert_eq!(aho_corasick_filter.filter("", "*"), "");

        aho_corasick_filter.insert("bat");
        assert_eq!(aho_corasick_filter.filter("bat", "*"), "***");
        assert_eq!(aho_corasick_filter.filter("batapple", "*"), "********");
        assert_eq!(aho_corasick_filter.filter("bataapple", "*"), "***a*****");
        assert_eq!(aho_corasick_filter.filter("batapapple", "*"), "***ap*****");

        aho_corasick_filter.insert("");
        assert_eq!(aho_corasick_filter.filter("apple", "*"), "*****");
        assert_eq!(aho_corasick_filter.filter("app", "*"), "***");
        assert_eq!(aho_corasick_filter.filter("bat", "*"), "***");
        assert_eq!(aho_corasick_filter.filter("batapple", "*"), "********");
        assert_eq!(aho_corasick_filter.filter("", "*"), "");

        //-----

        aho_corasick_filter = AhoCorasickFilter::new();
        aho_corasick_filter.insert("135");
        assert_eq!(aho_corasick_filter.filter_and_ignore_chars("123456", "*", HashSet::from(['2', '4', '6'])), "*2*4*6");
    }
}
