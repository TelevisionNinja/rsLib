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
            assert!(text.contains(&piece));
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

    #[test]
    fn pre_tokenize_basic_sentence() {
        let input = b"hello world!";
        let spans = ByteLevelBytePairEncoder::pre_tokenize(input);
        let pieces: Vec<&[u8]> = spans
            .iter()
            .map(|&(start, end)| &input[start..end])
            .collect();

        assert_eq!(pieces, vec![
                b"hello".as_ref(),
                b" world".as_ref(),
                b"!".as_ref()
            ]
        );
    }

    #[test]
    fn pre_tokenize_multiple_words() {
        let input = b"the quick brown fox";
        let spans = ByteLevelBytePairEncoder::pre_tokenize(input);
        let pieces: Vec<&[u8]> = spans
            .iter()
            .map(|&(start, end)| &input[start..end])
            .collect();

        assert_eq!(pieces, vec![
                b"the".as_ref(),
                b" quick".as_ref(),
                b" brown".as_ref(),
                b" fox".as_ref()
            ]
        );
    }

    #[test]
    fn pre_tokenize_punctuation() {
        let input = b"hello, world!";
        let spans = ByteLevelBytePairEncoder::pre_tokenize(input);
        let pieces: Vec<&[u8]> = spans
            .iter()
            .map(|&(start, end)| &input[start..end])
            .collect();

        assert_eq!(pieces, vec![
                b"hello".as_ref(),
                b",".as_ref(),
                b" world".as_ref(),
                b"!".as_ref()
            ]
        );
    }

    #[test]
    fn pre_tokenize_leading_whitespace() {
        let input = b" hello";
        let spans = ByteLevelBytePairEncoder::pre_tokenize(input);
        let pieces: Vec<&[u8]> = spans
            .iter()
            .map(|&(start, end)| &input[start..end])
            .collect();

        assert_eq!(pieces, vec![
                b" hello".as_ref()
            ]
        );
    }

    #[test]
    fn pre_tokenize_multiple_spaces() {
        let input = b"hello   world";
        let spans = ByteLevelBytePairEncoder::pre_tokenize(input);
        let pieces: Vec<&[u8]> = spans
            .iter()
            .map(|&(start, end)| &input[start..end])
            .collect();

        assert_eq!(pieces, vec![
                b"hello".as_ref(),
                b"   world".as_ref()
            ]
        );
    }

    #[test]
    fn pre_tokenize_newline() {
        let input = b"hello\nworld";
        let spans = ByteLevelBytePairEncoder::pre_tokenize(input);
        let pieces: Vec<&[u8]> = spans
            .iter()
            .map(|&(start, end)| &input[start..end])
            .collect();

        assert_eq!(pieces, vec![
                b"hello".as_ref(),
                b"\nworld".as_ref()
            ]
        );
    }

    #[test]
    fn build_vocab_learns_common_pair() {
        let mut encoder = ByteLevelBytePairEncoder::new();
        let samples = vec![
            b"abab".to_vec(),
            b"abab".to_vec(),
            b"abab".to_vec()
        ];

        encoder.build_vocab(&samples, 1);

        assert_eq!(
            encoder.encode_map.get(&(b'a' as Token, b'b' as Token)),
            Some(&256)
        );

        assert_eq!(
            encoder.decode_map.get(&256),
            Some(&(b'a' as Token, b'b' as Token))
        );
    }

    #[test]
    fn visualize_preserves_text() {
        let mut encoder = ByteLevelBytePairEncoder::new();
        let samples = vec![
            b"hello world".to_vec(),
            b"hello world".to_vec(),
            b"hello world".to_vec()
        ];

        encoder.build_vocab(&samples, 100);

        let pieces = encoder.visualize("hello world");
        let reconstructed: String = pieces.iter().cloned().collect();

        assert_eq!(reconstructed, "hello world");
    }

    #[test]
    fn visualize_has_space_with_following_word() {
        let mut encoder = ByteLevelBytePairEncoder::new();
        let samples = vec![
            b"hello world".to_vec(),
            b"hello world".to_vec(),
            b"hello world".to_vec()
        ];

        encoder.build_vocab(&samples, 100);

        let pieces = encoder.visualize("hello world");

        assert!(pieces
            .iter()
            .any(|piece| {
                piece.starts_with(' ')
            })
        );
    }

    #[test]
    fn encode_decode() {
        let mut encoder = ByteLevelBytePairEncoder::new();
        let samples = vec![
            b"hello world".to_vec(),
            b"the quick brown fox".to_vec(),
            b"hello there".to_vec()
        ];

        encoder.build_vocab(&samples, 100);

        let input = b"hello world! the quick fox";
        let tokens = encoder.encode(input);
        let decoded = encoder.decode(&tokens);

        assert_eq!(decoded, input);
    }

    #[test]
    fn recursive_and_non_recursive_decode_match() {
        let mut encoder = ByteLevelBytePairEncoder::new();
        let samples = vec![
            b"hello world".to_vec(),
            b"hello world".to_vec(),
            b"hello world".to_vec()
        ];

        encoder.build_vocab(&samples, 100);

        let tokens = encoder.encode(b"hello world");
        let recursive = encoder.decode_recursive(&tokens);
        let non_recursive = encoder.decode(&tokens);

        assert_eq!(recursive, non_recursive);
    }
}
