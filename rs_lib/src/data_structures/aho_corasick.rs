use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;

pub struct AhoCorasickNode {
    pub children: HashMap<char, usize>,
    pub suffix_link: Option<usize>,
    pub output_links: HashSet<usize>,
    pub length: usize,
}

impl AhoCorasickNode {
    pub fn new() -> Self {
        Self {
            children: HashMap::new(),
            suffix_link: None,
            output_links: HashSet::new(),
            length: 0
        }
    }
}

pub struct AhoCorasick {
    pub nodes: HashMap<usize, AhoCorasickNode>,
    pub root: usize
}

impl AhoCorasick {
    pub fn new() -> Self {
        let mut nodes = HashMap::new();
        let root = 0;
        nodes.insert(root, AhoCorasickNode::new());

        Self { nodes, root }
    }

    fn delete_trie_node(&mut self, node_id: usize, word: &Vec<char>, depth: usize) -> bool {
        if depth == word.len() {
            if self.nodes.get(&node_id).unwrap().length == 0 {
                return false;
            }

            self.nodes.get_mut(&node_id).unwrap().length = 0;
            return self.nodes.get(&node_id).unwrap().children.is_empty();
        }

        let c = word[depth];

        if !self.nodes.get(&node_id).unwrap().children.contains_key(&c) {
            return false;
        }

        let child_id = self.nodes.get(&node_id).unwrap().children.get(&c).unwrap();
        let should_delete_child = self.delete_trie_node(*child_id, word, depth + 1);

        if should_delete_child {
            self.nodes.get_mut(&node_id).unwrap().children.remove(&c).unwrap();
            return self.nodes.get(&node_id).unwrap().children.is_empty() && self.nodes.get(&node_id).unwrap().length == 0;
        }

        false
    }

    pub fn build(&mut self, words: Vec<&str>) {
        for word in words {
            self.insert(word);
        }

        self.construct_links();
    }

    fn insert_trie(&mut self, word: &str) {
        let mut node_id = self.root;

        for c in word.chars() {
            if !self.nodes.get(&node_id).unwrap().children.contains_key(&c) {
                let new_id = self.nodes.len();
                self.nodes.insert(new_id, AhoCorasickNode::new());
                self.nodes.get_mut(&node_id).unwrap().children.insert(c, new_id);
            }

            node_id = self.nodes.get(&node_id).unwrap().children[&c];
        }

        let new_node = self.nodes.get_mut(&node_id).unwrap();
        new_node.output_links.insert(node_id);
        new_node.length = word.len(); // height at the node is the length of the string
    }

    pub fn insert(&mut self, word: &str) {
        self.insert_trie(word);
        self.construct_links();
    }

    pub fn search(&self, string: &str) -> Vec<(usize, usize)> {
        let mut node = self.root;
        let mut output = Vec::new();

        // empty string case
        // only the root's output set's size is check because the empty string has no length
        if !self.nodes.get(&node).unwrap().output_links.is_empty() {
            output.push((0, 0)); // no need to iterate through output links
        }

        let characters: Vec<_> = string.chars().collect();
        let mut i = 0;

        while i < characters.len() {
            let c = characters[i];

            if let Some(&next) = self.nodes.get(&node).unwrap().children.get(&c) {
                node = next;
                i += 1;

                for &out_node in &self.nodes.get(&node).unwrap().output_links {
                    let len = self.nodes.get(&out_node).unwrap().length;
                    output.push((i - len, len));
                }
            }
            else if node == self.root {
                i += 1;
            }
            else {
                node = self.nodes.get(&node).unwrap().suffix_link.unwrap();
            }
        }

        output
    }

    pub fn remove(&mut self, word: &str) {
        self.delete_trie_node(self.root, &word.chars().collect(), 0);
        self.delete_links();
        self.construct_links();
    }

    fn construct_links(&mut self) {
        // BFS
        let mut node_queue: VecDeque<usize> = VecDeque::new();

        for (&_key, &value) in self.nodes.get(&self.root).unwrap().children.clone().iter() {
            self.nodes.get_mut(&value).unwrap().suffix_link = Some(self.root);
            node_queue.push_back(value);
        }

        while let Some(current) = node_queue.pop_front() {
            let current_node = &mut self.nodes.get(&current).unwrap();
            let keys: Vec<char> = current_node.children.keys().cloned().collect();

            for key in keys {
                let child_id = self.nodes.get(&current).unwrap().children[&key];
                node_queue.push_back(child_id);

                // output links
                let mut failure_node_id = self.nodes.get(&current).unwrap().suffix_link;

                while let Some(current_failure_node_id) = failure_node_id {
                    if self.nodes.get(&current_failure_node_id).unwrap().children.contains_key(&key) {
                        break;
                    }

                    failure_node_id = self.nodes.get(&current_failure_node_id).unwrap().suffix_link;
                }

                if let Some(failure_node_id_unrapped) = failure_node_id {
                    self.nodes.get_mut(&child_id).unwrap().suffix_link = self.nodes.get(&failure_node_id_unrapped).unwrap().children.get(&key).copied();
                }
                else {
                    self.nodes.get_mut(&child_id).unwrap().suffix_link = Some(self.root);
                }

                let suffix_node_id = self.nodes.get(&child_id).unwrap().suffix_link.unwrap();
                let output_clone = self.nodes.get(&suffix_node_id).unwrap().output_links.clone();
                for output_node in output_clone {
                    self.nodes.get_mut(&child_id).unwrap().output_links.insert(output_node);
                }
            }
        }
    }

    fn delete_links(&mut self) {
        // DFS because of stack implementation time complexity. traversal order does not matter
        let mut stack = vec![self.root];

        while let Some(id) = stack.pop() {
            let aho_corasick_node = self.nodes.get_mut(&id).unwrap();

            aho_corasick_node.suffix_link = None;
            aho_corasick_node.output_links.clear();

            if aho_corasick_node.length != 0 {
                aho_corasick_node.output_links.insert(id);
            }

            for &child in aho_corasick_node.children.values() {
                stack.push(child);
            }
        }
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
        let mut aho_corasick = AhoCorasick::new();

        assert_eq!(0, aho_corasick.nodes.len() - 1);
        assert_eq!(true, aho_corasick.nodes.get(&0).unwrap().children.is_empty());
        assert_eq!(true, aho_corasick.nodes.get(&0).unwrap().output_links.is_empty());
        assert_eq!(None, aho_corasick.nodes.get(&0).unwrap().suffix_link);

        assert_eq!(vector_pair_to_string(aho_corasick.search("")), "");
        assert_eq!(vector_pair_to_string(aho_corasick.search("apple")), "");

        let word_list = vec!["apple", "app", "bat"];
        aho_corasick.build(word_list);

        assert_eq!(vector_pair_to_string(aho_corasick.search("apple")), "(0 3), (0 5)");
        assert_eq!(vector_pair_to_string(aho_corasick.search("app")), "(0 3)");
        assert_eq!(vector_pair_to_string(aho_corasick.search("bat")), "(0 3)");
        assert_eq!(vector_pair_to_string(aho_corasick.search("batapple")), "(0 3), (3 3), (3 5)");

        aho_corasick.remove("apple");
        assert_eq!(vector_pair_to_string(aho_corasick.search("apple")), "(0 3)");
        assert_eq!(vector_pair_to_string(aho_corasick.search("app")), "(0 3)");

        aho_corasick.remove("app");
        assert_eq!(vector_pair_to_string(aho_corasick.search("app")), "");
        assert_eq!(vector_pair_to_string(aho_corasick.search("bat")), "(0 3)");

        aho_corasick.insert("apple");
        aho_corasick.insert("app");
        assert_eq!(vector_pair_to_string(aho_corasick.search("apple")), "(0 3), (0 5)");
        assert_eq!(vector_pair_to_string(aho_corasick.search("app")), "(0 3)");

        assert_eq!(vector_pair_to_string(aho_corasick.search("")), "");
        aho_corasick.remove("bat");
        assert_eq!(vector_pair_to_string(aho_corasick.search("")), "");

        aho_corasick.insert("");
        assert_eq!(vector_pair_to_string(aho_corasick.search("")), "(0 0)");
        aho_corasick.remove("");
        assert_eq!(vector_pair_to_string(aho_corasick.search("")), "");

        aho_corasick.insert("i");
        aho_corasick.insert("in");
        aho_corasick.insert("tin");
        aho_corasick.insert("sting");
        assert_eq!(
            vector_pair_to_string(aho_corasick.search("stings")) == "(2 1), (1 3), (2 2), (0 5)" ||
            vector_pair_to_string(aho_corasick.search("stings")) == "(2 1), (2 2), (1 3), (0 5)",
            true
        );
    }
}
