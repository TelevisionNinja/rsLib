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

const EIGHT_BIT_MAX: Token = 1 << 8; // 256
const SEVEN_BIT_MAX: u8 = 1 << 7 ; // 128

impl ByteLevelBytePairEncoder {
    pub fn new() -> Self {
        Self {
            encode_map: HashMap::new(),
            decode_map: HashMap::new(),
            next_token: EIGHT_BIT_MAX
        }
    }

    fn pre_tokenize(bytes: &[u8]) -> Vec<(usize, usize)> {
        let mut spans = Vec::new();
        let mut i = 0;

        while i < bytes.len() {
            let start = i;

            if bytes[i].is_ascii_whitespace() { // whitespace
                while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                    i += 1;
                }

                // attach whitespace to following text
                while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] >= SEVEN_BIT_MAX) {
                    i += 1;
                }
            }
            else if bytes[i].is_ascii_alphanumeric() || bytes[i] >= SEVEN_BIT_MAX { // text
                while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] >= SEVEN_BIT_MAX) {
                    i += 1;
                }
            }
            else { // symbols
                i += 1;
            }

            spans.push((start, i));
        }

        spans
    }

    fn merge_pair(&self, input: &[Token], pair: (Token, Token), merge_token: Token) -> Vec<Token> {
        let mut out = Vec::with_capacity(input.len());
        let mut i = 0;

        while i < input.len() {
            if i + 1 < input.len() && input[i] == pair.0 && input[i + 1] == pair.1 {
                out.push(merge_token);
                i += 1;
            }
            else {
                out.push(input[i]);
            }

            i += 1;
        }

        out
    }

    pub fn build_vocab(&mut self, samples: &[Vec<u8>], num_merges: usize) {
        let mut corpus: Vec<Vec<Token>> = Vec::new();

        for sample in samples {
            let spans = Self::pre_tokenize(sample);

            for &(start, end) in &spans {
                if start == end {
                    continue;
                }

                let mut sequence = Vec::with_capacity(end - start);

                for &byte in &sample[start..end] {
                    sequence.push(byte as Token);
                }

                corpus.push(sequence);
            }
        }

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

    fn encode_piece(&self, input: &[u8]) -> Vec<Token> {
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

    pub fn encode(&self, input: &[u8]) -> Vec<Token> {
        let spans = Self::pre_tokenize(input);
        let mut tokens = Vec::new();

        for &(start, end) in &spans {
            let piece = self.encode_piece(&input[start..end]);
            tokens.extend(piece);
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
        if token < EIGHT_BIT_MAX {
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
                if token < EIGHT_BIT_MAX {
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

    fn encode_piece_with_spans(&self, bytes: &[u8]) -> Vec<TokenSpan> {
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

    pub fn encode_with_spans(&self, bytes: &[u8]) -> Vec<TokenSpan> {
        let spans = Self::pre_tokenize(bytes);
        let mut tokens = Vec::new();

        for &(start, end) in &spans {
            let piece = self.encode_piece_with_spans(&bytes[start..end]);

            for mut token in piece {
                token.start += start;
                token.end += start;

                tokens.push(token);
            }
        }

        tokens
    }

    pub fn visualize(&self, input: &str) -> Vec<String> {
        self.encode_with_spans(input.as_bytes())
            .into_iter()
            .map(|span| {
                String::from_utf8_lossy(
                    &input.as_bytes()[span.start..span.end]
                )
                .into_owned()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    include!("byte_level_byte_pair_encoder_tests.rs");

    #[test]
    fn merge_pair_does_not_drop_last_token() {
        let encoder = ByteLevelBytePairEncoder::new();
        let input = vec![
            1,
            2,
            3,
            4
        ];
        let result =
            encoder.merge_pair(
                &input,
                (2, 3),
                256
            );

        assert_eq!(result, vec![1, 256, 4]);
    }

    #[test]
    fn merge_pair_at_end() {
        let encoder = ByteLevelBytePairEncoder::new();
        let input = vec![
            1,
            2,
            3
        ];
        let result = encoder.merge_pair(
            &input,
            (2, 3),
            256
        );

        assert_eq!(result, vec![1, 256]);
    }
}
