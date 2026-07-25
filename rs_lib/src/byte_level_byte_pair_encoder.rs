use std::collections::HashMap;

pub type Token = u64;

pub struct ByteLevelBytePairEncoder {
    encode_map: HashMap<(Token, Token), Token>,
    decode_map: HashMap<Token, (Token, Token)>,
    next_token: Token
}

#[derive(Clone)]
pub struct TokenSpan {
    pub token: Token,
    pub start: usize,
    pub end: usize // exclusive
}

impl ByteLevelBytePairEncoder {
    pub fn new() -> Self {
        Self {
            encode_map: HashMap::new(),
            decode_map: HashMap::new(),
            next_token: 256
        }
    }

    fn merge_pair(&self, input: &[Token], pair: (Token, Token), merge_token: Token) -> Vec<Token> {
        let mut out = Vec::with_capacity(input.len());

        for input_pair in input.windows(2) {
            if input_pair[0] == pair.0 && input_pair[1] == pair.1 {
                out.push(merge_token);
            }
            else {
                out.push(input_pair[0]);
            }
        }

        out
    }

    pub fn build_vocab(&mut self, samples: &[Vec<u8>], num_merges: usize) {
        let mut corpus: Vec<Vec<Token>> = samples
            .iter()
            .map(|s| s
                .iter()
                .map(|b| *b as Token)
                .collect()
            )
            .collect();

        for _ in 0..num_merges {
            let mut pair_counts: HashMap<(Token, Token), usize> = HashMap::new();

            for seq in &corpus {
                for pair in seq.windows(2) {
                    *pair_counts.entry((pair[0], pair[1])).or_default() += 1;
                }
            }

            let max_result = pair_counts.iter().max_by_key(|(_, c)| *c);
            if max_result.is_none() {
                break;
            }

            let (&best_pair, &count) = max_result.unwrap();

            if count < 2 {
                break;
            }

            let new_token = self.next_token;
            self.next_token += 1;

            self.encode_map.insert(best_pair, new_token);
            self.decode_map.insert(new_token, best_pair);

            for seq in corpus.iter_mut() {
                *seq = self.merge_pair(seq, best_pair, new_token);
            }
        }
    }

    pub fn encode(&self, input: &[u8]) -> Vec<Token> {
        let mut tokens: Vec<Token> = input
            .iter()
            .map(|b| *b as Token)
            .collect();

        loop {
            let mut changed = false;
            let mut out: Vec<Token> = Vec::with_capacity(tokens.len());
            let mut i = 0;

            while i < tokens.len() {
                if i + 1 < tokens.len() {
                    if let Some(&merged) = self.encode_map.get(&(tokens[i], tokens[i + 1])) {
                        out.push(merged);
                        i += 2;
                        changed = true;
                        continue;
                    }
                }

                out.push(tokens[i]);
                i += 1;
            }

            if !changed {
                break;
            }

            tokens = out;
        }

        tokens
    }

    pub fn decode_recursive(&self, tokens: &[Token]) -> Vec<u8> {
        let mut out: Vec<u8> = Vec::new();

        for &token in tokens {
            self.expand_recursive(token, &mut out);
        }

        out
    }

    fn expand_recursive(&self, token: Token, out: &mut Vec<u8>) {
        if token < 256 {
            out.push(token as u8);
        }
        else {
            let &(a, b) = self.decode_map.get(&token).unwrap();
            self.expand_recursive(a, out);
            self.expand_recursive(b, out);
        }
    }

    pub fn decode(&self, tokens: &[Token]) -> Vec<u8> {
        let mut out = Vec::new();
        let mut stack = Vec::new();

        for &token in tokens {
            stack.push(token);

            while let Some(token) = stack.pop() {
                if token < 256 {
                    out.push(token as u8);
                }
                else {
                    // max stack size is the depth of the merges
                    let &(left, right) = self.decode_map.get(&token).unwrap();

                    stack.push(right);
                    stack.push(left);
                }
            }
        }

        out
    }

    pub fn encode_with_spans(&self, bytes: &[u8]) -> Vec<TokenSpan> {
        let mut token_spans: Vec<TokenSpan> = bytes
            .iter()
            .enumerate()
            .map(|(i, &b)| TokenSpan {
                token: b as Token,
                start: i,
                end: i + 1
            })
            .collect();

        loop {
            let mut changed = false;
            let mut out: Vec<TokenSpan> = Vec::with_capacity(token_spans.len());
            let mut i = 0;

            while i < token_spans.len() {
                if i + 1 < token_spans.len() {
                    if let Some(&merged) = self.encode_map.get(&(token_spans[i].token, token_spans[i + 1].token)) {
                        out.push(TokenSpan {
                            token: merged,
                            start: token_spans[i].start,
                            end: token_spans[i + 1].end
                        });

                        i += 2;
                        changed = true;
                        continue;
                    }
                }

                out.push(token_spans[i].clone());
                i += 1;
            }

            if !changed {
                break;
            }

            token_spans = out;
        }

        token_spans
    }

    pub fn visualize<'a>(&self, input: &'a str) -> Vec<&'a str> {
        self.encode_with_spans(input.as_bytes())
            .into_iter()
            .map(|span| &input[span.start..span.end])
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_identity() {
        let samples = vec![
            b"hello hello hello".to_vec(),
            b"general kenobi".to_vec()
        ];
        let mut bpe = ByteLevelBytePairEncoder::new();
        bpe.build_vocab(&samples, 50);
        let input = b"hello general";
        let encoded = bpe.encode(input);
        let decoded = bpe.decode(&encoded);

        assert_eq!(decoded, input);
    }

    #[test]
    fn build_vocab_test() {
        let samples = vec![
            b"aaaaaa".to_vec()
        ];
        let mut bpe = ByteLevelBytePairEncoder::new();
        bpe.build_vocab(&samples, 10);
        let encoded = bpe.encode(b"aaaaaa");

        assert!(encoded.len() < 6);
    }

    #[test]
    fn binary_data() {
        let samples = vec![
            vec![0,1,2,3,4,5,6,7,8,9],
            vec![0,1,2,3,4,5]
        ];
        let mut bpe = ByteLevelBytePairEncoder::new();
        bpe.build_vocab(&samples, 20);
        let bytes = vec![0,1,2,3,4,5,6,7];
        let enc = bpe.encode(&bytes);
        let dec = bpe.decode(&enc);

        assert_eq!(bytes, dec);
    }

    #[test]
    fn repeated_pattern() {
        let samples = vec![
            b"abcabcabcabc".to_vec()
        ];
        let mut bpe = ByteLevelBytePairEncoder::new();
        bpe.build_vocab(&samples, 20);
        let enc = bpe.encode(b"abcabcabcabc");

        assert!(enc.len() < 12);

        let dec = bpe.decode(&enc);

        assert_eq!(dec, b"abcabcabcabc");
    }

    #[test]
    fn empty_input() {
        let samples = vec![
            b"abc".to_vec()
        ];
        let mut bpe = ByteLevelBytePairEncoder::new();
        bpe.build_vocab(&samples, 5);
        let enc = bpe.encode(b"");

        assert!(enc.is_empty());

        let dec = bpe.decode(&enc);

        assert!(dec.is_empty());
    }

    #[test]
    fn single_byte() {
        let samples = vec![
            b"aaaa".to_vec()
        ];
        let mut bpe = ByteLevelBytePairEncoder::new();
        bpe.build_vocab(&samples, 10);
        let enc = bpe.encode(b"a");

        assert_eq!(enc.len(), 1);

        let dec = bpe.decode(&enc);

        assert_eq!(dec, b"a");
    }

    #[test]
    fn no_vocab_build() {
        let bpe = ByteLevelBytePairEncoder::new();
        let text = b"abcdef";
        let enc = bpe.encode(text);

        assert_eq!(enc.len(), text.len());

        let dec = bpe.decode(&enc);

        assert_eq!(dec, text);
    }

    #[test]
    fn spans_cover_entire_input() {
        let mut bpe = ByteLevelBytePairEncoder::new();
        bpe.build_vocab(
            &[b"hello hello".to_vec()],
            20,
        );
        let input = b"hello hello";
        let spans = bpe.encode_with_spans(input);

        assert!(!spans.is_empty());
        assert_eq!(spans.first().unwrap().start, 0);
        assert_eq!(spans.last().unwrap().end, input.len());

        for pair in spans.windows(2) {
            assert_eq!(pair[0].end, pair[1].start);
        }
    }

    #[test]
    fn spans_reconstruct_original_bytes() {
        let mut bpe = ByteLevelBytePairEncoder::new();
        bpe.build_vocab(
            &[b"the quick brown fox".to_vec()],
            50
        );
        let input = b"the quick brown fox";
        let spans = bpe.encode_with_spans(input);
        let reconstructed: Vec<u8> = spans
            .iter()
            .flat_map(|s| input[s.start..s.end].iter().copied())
            .collect();

        assert_eq!(reconstructed, input);
    }

    #[test]
    fn encode_matches_token_spans() {
        let mut bpe = ByteLevelBytePairEncoder::new();
        bpe.build_vocab(
            &[b"banana banana".to_vec()],
            30
        );
        let input = b"banana";
        let tokens = bpe.encode(input);
        let span_tokens: Vec<Token> = bpe
            .encode_with_spans(input)
            .into_iter()
            .map(|s| s.token)
            .collect();

        assert_eq!(tokens, span_tokens);
    }

    #[test]
    fn visualize_matches_original_string() {
        let mut bpe = ByteLevelBytePairEncoder::new();
        bpe.build_vocab(
            &[b"hello world".to_vec()],
            30
        );
        let text = "hello world";
        let pieces = bpe.visualize(text);

        assert_eq!(pieces.concat(), text);
    }

    #[test]
    fn visualize_returns_substrings() {
        let mut bpe = ByteLevelBytePairEncoder::new();
        bpe.build_vocab(
            &[b"mississippi".to_vec()],
            20
        );
        let text = "mississippi";

        for piece in bpe.visualize(text) {
            assert!(text.contains(piece));
        }
    }

    #[test]
    fn empty_input_has_no_spans() {
        let bpe = ByteLevelBytePairEncoder::new();

        assert!(bpe.encode_with_spans(b"").is_empty());
        assert!(bpe.visualize("").is_empty());
    }

    #[test]
    fn single_byte_span() {
        let bpe = ByteLevelBytePairEncoder::new();
        let spans = bpe.encode_with_spans(b"x");

        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].start, 0);
        assert_eq!(spans[0].end, 1);
        assert_eq!(spans[0].token, b'x' as Token);
    }

    #[test]
    fn decode_round_trip_with_spans() {
        let mut bpe = ByteLevelBytePairEncoder::new();
        bpe.build_vocab(
            &[b"abracadabra".to_vec()],
            40
        );
        let input = b"abracadabra";
        let spans = bpe.encode_with_spans(input);
        let tokens: Vec<_> = spans.iter().map(|s| s.token).collect();
        let decoded = bpe.decode(&tokens);

        assert_eq!(decoded, input);
    }
}
