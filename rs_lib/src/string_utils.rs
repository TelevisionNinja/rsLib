use std::collections::HashSet;
use std::cmp::min;

pub fn find(string: &str, substring: &str, index: usize) -> Option<usize> {
    string.get(index..)
        .and_then(|s|
            s.find(substring)
            .map(|i| i + index)
        )
}

pub fn rfind(string: &str, substring: &str, index: usize) -> Option<usize> {
    string.get(..index)
        .and_then(|s|
            s.rfind(substring)
        )
}

pub fn url_parameter_filter(url:&str, parameters_vec: Vec<&str>, is_blacklist: bool) -> String {
    // domain part

    let first_split = url.split_once("?");

    if first_split.is_none() {
        return url.to_string();
    }

    let domain_and_query = first_split.unwrap();
    let domain_part = domain_and_query.0;
    let query_part = domain_and_query.1;
    let mut result = String::with_capacity(url.len());
    result.push_str(domain_part);
    let parameters: HashSet<&str> = HashSet::from_iter(parameters_vec);

    //---------------------
    // parameter part

    let parameter_iterator = query_part.split("&");
    let mut filtered_parameters = parameter_iterator.filter(
        |parameter| {
            let current_query_key = parameter
                .split_once("=")
                .map_or(
                    *parameter,
                    |(query_key, _)|
                        query_key
                );

            is_blacklist ^ parameters.contains(current_query_key)
        }
    );

    let first_parameter = filtered_parameters.next();

    if first_parameter.is_some() {
        result.push_str("?");
        result.push_str(first_parameter.unwrap());

        for parameter in filtered_parameters {
            result.push_str("&");
            result.push_str(parameter);
        }
    }

    result
}

pub fn word_n_grams(array: Vec<&str>, n: usize) -> HashSet<String> {
    let mut set = HashSet::new();

    if array.len() < n {
        return set;
    }

    for window in array.windows(n) {
        set.insert(window.join(" "));
    }

    set
}

pub fn char_n_grams(s: &str, n: usize) -> HashSet<String> {
    let mut set = HashSet::new();
    let chars: Vec<char> = s.chars().collect();

    if chars.len() < n {
        return set;
    }

    for window in chars.windows(n) {
        set.insert(window.iter().collect());
    }

    set
}

pub mod youtube {
    pub fn is_youtube_url(url: &str) -> bool {
        let starting_urls = [
            "https://www.youtube.com/",
            "https://youtube.com/",
            "https://youtu.be/"
        ];

        for starting_url in starting_urls {
            if url.starts_with(starting_url) {
                return true;
            }
        }

        false
    }

    pub fn short_to_video(url: &str) -> String {
        if is_youtube_url(url) {
            let path = "shorts/";
            let index = url.find(path);

            if index.is_some() {
                return url.replace(path, "watch?v=");
            }
        }

        url.to_string()
    }

    pub fn remove_tracking_parameters(url: &str) -> String {
        if is_youtube_url(url) {
            return super::url_parameter_filter(url, vec!["si", "pp", "is"], true);
        }

        url.to_string()
    }
}

pub fn edit_distance(string1: &str, string2: &str) -> usize {
    let word1: Vec<_> = string1.chars().collect();
    let word2: Vec<_> = string2.chars().collect();
    let word1_length = word1.len();
    let word2_length = word2.len();

    if word1_length == 0 {
        return word2_length;
    }
    else if word2_length == 0 {
        return word1_length;
    }

    let mut distance: Vec<usize> = Vec::with_capacity(word2_length);

    for i in 1..=word2_length {
        distance.push(i);
    }

    /*
        j ->
    i   w o r d 2
    | w
    v o
      r
      d
      1

    left = delete
    corner = replace
    top = insert
    */
    for i in 0..word1_length {
        let current_char = word1[i];
        let mut j = 0;
        let mut replace = i;
        let mut insert = distance[j];

        // compute the first element of the array
        if current_char == word2[j] {
            distance[j] = replace;
        }
        else {
            let delete_char = i + 1;
            distance[j] = 1 + min(min(delete_char, replace), insert);
        }

        j += 1;

        // compute the rest of the array
        while j < word2_length {
            replace = insert;
            insert = distance[j];

            if current_char == word2[j] {
                distance[j] = replace;
            }
            else {
                let delete_char = distance[j - 1];
                distance[j] = 1 + min(min(delete_char, replace), insert);
            }

            j += 1;
        }
    }

    return distance[word2_length - 1];
}

// ---------------------------

pub struct ApproximateMatch {
    pub start: usize,
    pub end: usize,
    pub distance: usize,
    pub matched_string: String
}

#[derive(Clone)]
enum EditType {
    SUBSTITUTION,
    DELETION,
    INSERTION
}

pub fn approximate_match(pattern: &str, text: &str) -> ApproximateMatch {
    let pattern_array: Vec<char> = pattern.chars().collect();
    let text_array: Vec<char> = text.chars().collect();
    let m = pattern_array.len();
    let n = text_array.len();

    if m == 0 {
        return ApproximateMatch {
            start: 0,
            end: 0,
            distance: 0,
            matched_string: "".to_string()
        };
    }

    if n == 0 {
        return ApproximateMatch {
            start: 0,
            end: 0,
            distance: m,
            matched_string: "".to_string()
        };
    }

    let mut dp: Vec<Vec<usize>> = vec![vec![0; n + 1]; m + 1];

    // remember which operation to trace the match backwards
    let mut path: Vec<Vec<EditType>> = vec![vec![EditType::SUBSTITUTION; n + 1]; m + 1];

    // first row is initialized to zero so that matching can begin at any position
    for j in 0..=n {
        dp[0][j] = 0;
    }

    for i in 1..=m {
        dp[i][0] = i;
        path[i][0] = EditType::DELETION;
    }

    for i in 1..=m {
        for j in 1..=n {
            let mut substitution= dp[i - 1][j - 1];

            if pattern_array[i - 1] != text_array[j - 1] {
                substitution += 1;
            }

            let deletion = dp[i - 1][j] + 1;
            let insertion = dp[i][j - 1] + 1;

            let value;
            let operation;
            if substitution <= deletion && substitution <= insertion {
                value = substitution;
                operation = EditType::SUBSTITUTION;
            }
            else if deletion <= insertion {
                value = deletion;
                operation = EditType::DELETION;
            }
            else {
                value = insertion;
                operation = EditType::INSERTION;
            }

            dp[i][j] = value;
            path[i][j] = operation;
        }
    }

    // find the minimum value in last row
    let mut end = 0;
    let mut best_distance = dp[m][0];

    for j in 1..=n {
        if dp[m][j] <= best_distance { // rightmost
            best_distance = dp[m][j];
            end = j;
        }
    }

    // trace backwards. the column ended on is where the matching substring begins
    let mut i = m;
    let mut start = end;

    while i > 0 {
        match path[i][start] {
            // Diagonal: substitution
            EditType::SUBSTITUTION => {
                i -= 1;
                start -= 1;
            }

            // Up: deletion
            EditType::DELETION => {
                i -= 1;
            }

            // Left: insertion
            EditType::INSERTION => {
                start -= 1;
            }
        }
    }

    ApproximateMatch {
        start,
        end,
        distance: best_distance,
        matched_string: text_array[start..end].iter().collect()
    }
}

// ---------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_tests() {
        assert_eq!(Some(0), find("", "", 0));
        assert_eq!(Some(0), find("abc", "a", 0));
        assert_eq!(None, find("abc", "a", 1));
        assert_eq!(Some(2), find("abc", "c", 0));
        assert_eq!(Some(2), find("abc", "c", 1));
        assert_eq!(Some(2), find("abc", "c", 2));
        assert_eq!(None, find("abc", "c", 3));
    }

    #[test]
    fn rfind_tests() {
        assert_eq!(Some(0), rfind("", "", 0));
        assert_eq!(None, rfind("abc", "a", 0));
        assert_eq!(Some(0), rfind("abc", "a", 1));
        assert_eq!(None, rfind("abc", "c", 0));
        assert_eq!(None, rfind("abc", "c", 1));
        assert_eq!(None, rfind("abc", "c", 2));
        assert_eq!(Some(2), rfind("abc", "c", 3));
    }

    #[test]
    fn youtube_tests() {
        assert_eq!("https://youtu.be/2BO83Ig-E8E", youtube::short_to_video("https://youtu.be/2BO83Ig-E8E"));
        assert_eq!("https://youtube.com/watch?v=60gZOXu5gcQ?si=EkAu2o2eUgp4SZV-", youtube::short_to_video("https://youtube.com/shorts/60gZOXu5gcQ?si=EkAu2o2eUgp4SZV-"));
        assert_eq!("https://www.youtube.com/watch?v=H3O6-SHr2fc", youtube::short_to_video("https://www.youtube.com/shorts/H3O6-SHr2fc"));
        assert_eq!("https://www.youtube.com/watch?v=HCE_lFUMXNg", youtube::short_to_video("https://www.youtube.com/watch?v=HCE_lFUMXNg"));

        assert_eq!("https://youtu.be/2BO83Ig-E8E", youtube::remove_tracking_parameters("https://youtu.be/2BO83Ig-E8E"));
        assert_eq!("https://youtube.com/shorts/60gZOXu5gcQ", youtube::remove_tracking_parameters("https://youtube.com/shorts/60gZOXu5gcQ?si=EkAu2o2eUgp4SZV-"));
        assert_eq!("https://www.youtube.com/shorts/H3O6-SHr2fc", youtube::remove_tracking_parameters("https://www.youtube.com/shorts/H3O6-SHr2fc"));
        assert_eq!("https://www.youtube.com/watch?v=HCE_lFUMXNg", youtube::remove_tracking_parameters("https://www.youtube.com/watch?si=EkAu2o2eUgp4SZV-&v=HCE_lFUMXNg"));
        assert_eq!("https://www.youtube.com/watch?v=HCE_lFUMXNg", youtube::remove_tracking_parameters("https://www.youtube.com/watch?v=HCE_lFUMXNg&si=EkAu2o2eUgp4SZV-"));

        assert_eq!("https://youtube.com/shorts/60gZOXu5gcQ", youtube::remove_tracking_parameters("https://youtube.com/shorts/60gZOXu5gcQ?pp=EkAu2o2eUgp4SZV-"));
        assert_eq!("https://www.youtube.com/watch?v=HCE_lFUMXNg", youtube::remove_tracking_parameters("https://www.youtube.com/watch?pp=EkAu2o2eUgp4SZV-&v=HCE_lFUMXNg"));
        assert_eq!("https://www.youtube.com/watch?v=vWLUMXNhWANg", youtube::remove_tracking_parameters("https://www.youtube.com/watch?v=vWLUMXNhWANg&pp=ygUhamltIGdyZWVuIGFu2o2eYW4gcmFuZ2VyIGJhcmVmb290"));

        assert_eq!("https://www.youtube.com/watch?v=HCE_lFUMXNg", youtube::remove_tracking_parameters("https://www.youtube.com/watch?si=EkAu2o2eUgp4SZV-&v=HCE_lFUMXNg&pp=EkAu2o2eUgp4SZV-"));
        assert_eq!("https://www.youtube.com/watch?v=HCE_lFUMXNg", youtube::remove_tracking_parameters("https://www.youtube.com/watch?v=HCE_lFUMXNg&pp=EkAu2o2eUgp4SZV-&si=EkAu2o2eUgp4SZV-"));
    }

    #[test]
    fn url_parameter_whitelist_tests() {
        assert_eq!("https://youtu.be/2BO83Ig-E8E", url_parameter_filter("https://youtu.be/2BO83Ig-E8E", vec!["v"], false));
        assert_eq!("https://youtube.com/shorts/60gZOXu5gcQ", url_parameter_filter("https://youtube.com/shorts/60gZOXu5gcQ?si=EkAu2o2eUgp4SZV-", vec!["v"], false));
        assert_eq!("https://www.youtube.com/shorts/H3O6-SHr2fc", url_parameter_filter("https://www.youtube.com/shorts/H3O6-SHr2fc", vec!["v"], false));
        assert_eq!("https://www.youtube.com/watch?v=HCE_lFUMXNg", url_parameter_filter("https://www.youtube.com/watch?v=HCE_lFUMXNg", vec!["v"], false));

        assert_eq!("https://www.youtube.com/watch?v=HCE_lFUMXNg", url_parameter_filter("https://www.youtube.com/watch?si=EkAu2o2eUgp4SZV-&v=HCE_lFUMXNg", vec!["v"], false));
        assert_eq!("https://www.youtube.com/watch?v=HCE_lFUMXNg", url_parameter_filter("https://www.youtube.com/watch?v=HCE_lFUMXNg&si=EkAu2o2eUgp4SZV-", vec!["v"], false));

        assert_eq!("https://youtube.com/shorts/60gZOXu5gcQ", url_parameter_filter("https://youtube.com/shorts/60gZOXu5gcQ?pp=EkAu2o2eUgp4SZV-", vec!["v"], false));
        assert_eq!("https://www.youtube.com/watch?v=HCE_lFUMXNg", url_parameter_filter("https://www.youtube.com/watch?pp=EkAu2o2eUgp4SZV-&v=HCE_lFUMXNg", vec!["v"], false));
        assert_eq!("https://www.youtube.com/watch?v=vWLUMXNhWANg", url_parameter_filter("https://www.youtube.com/watch?v=vWLUMXNhWANg&pp=ygUhamltIGdyZWVuIGFu2o2eYW4gcmFuZ2VyIGJhcmVmb290", vec!["v"], false));

        assert_eq!("https://www.youtube.com/watch?v=HCE_lFUMXNg", url_parameter_filter("https://www.youtube.com/watch?si=EkAu2o2eUgp4SZV-&v=HCE_lFUMXNg&pp=EkAu2o2eUgp4SZV-", vec!["v"], false));
        assert_eq!("https://www.youtube.com/watch?v=HCE_lFUMXNg", url_parameter_filter("https://www.youtube.com/watch?v=HCE_lFUMXNg&pp=EkAu2o2eUgp4SZV-&si=EkAu2o2eUgp4SZV-", vec!["v"], false));

        assert_eq!("https://www.youtube.com/watch?v=HCE_lFUMXNg", url_parameter_filter("https://www.youtube.com/watch?si EkAu2o2eUgp4SZV-&v=HCE_lFUMXNg&pp_EkAu2o2eUgp4SZV-", vec!["v"], false));
        assert_eq!("https://www.youtube.com/watch?si EkAu2o2eUgp4SZV-&pp_EkAu2o2eUgp4SZV-", url_parameter_filter("https://www.youtube.com/watch?si EkAu2o2eUgp4SZV-&v=HCE_lFUMXNg&pp_EkAu2o2eUgp4SZV-", vec!["v"], true));

        assert_eq!("https://www.youtube.com/watch?v=HCE_lFUMXNg", url_parameter_filter("https://www.youtube.com/watch?si EkAu2o2eUgp4SZV-&v=HCE_lFUMXNg&pp_EkAu2o2eUgp4SZV-&a", vec!["v"], false));
        assert_eq!("https://www.youtube.com/watch?si EkAu2o2eUgp4SZV-&pp_EkAu2o2eUgp4SZV-&a", url_parameter_filter("https://www.youtube.com/watch?si EkAu2o2eUgp4SZV-&v=HCE_lFUMXNg&pp_EkAu2o2eUgp4SZV-&a", vec!["v"], true));
        assert_eq!("https://www.youtube.com/watch?v=HCE_lFUMXNg", url_parameter_filter("https://www.youtube.com/watch?si EkAu2o2eUgp4SZV-&v=HCE_lFUMXNg&pp_EkAu2o2eUgp4SZV-&a=", vec!["v"], false));
        assert_eq!("https://www.youtube.com/watch?si EkAu2o2eUgp4SZV-&pp_EkAu2o2eUgp4SZV-&a=", url_parameter_filter("https://www.youtube.com/watch?si EkAu2o2eUgp4SZV-&v=HCE_lFUMXNg&pp_EkAu2o2eUgp4SZV-&a=", vec!["v"], true));
    }

    #[test]
    fn char_n_grams_test() {
        let grams = char_n_grams("test", 2);
        let expected: HashSet<String> = ["te", "es", "st"].iter().map(|s| s.to_string()).collect();
        assert_eq!(grams, expected);
    }

    #[test]
    fn char_n_grams_short() {
        let grams = char_n_grams("hi", 3);
        assert!(grams.is_empty());
    }

    #[test]
    fn word_n_grams_test() {
        let grams = word_n_grams(vec!["apple", "app", "bat", "apap"], 2);
        let expected: HashSet<String> = ["apple app", "app bat", "bat apap"].iter().map(|s| s.to_string()).collect();
        assert_eq!(grams, expected);
    }

    #[test]
    fn word_n_grams_short() {
        let grams = word_n_grams(vec!["hi"], 3);
        assert!(grams.is_empty());
    }

    #[test]
    fn identical_strings() {
        assert_eq!(edit_distance("", ""), 0);
        assert_eq!(edit_distance("hello", "hello"), 0);
    }

    #[test]
    fn empty_strings() {
        assert_eq!(edit_distance("", "abc"), 3);
        assert_eq!(edit_distance("abc", ""), 3);
    }

    #[test]
    fn single_insertion() {
        assert_eq!(edit_distance("cat", "cats"), 1);
    }

    #[test]
    fn single_deletion() {
        assert_eq!(edit_distance("cats", "cat"), 1);
    }

    #[test]
    fn single_substitution() {
        assert_eq!(edit_distance("cat", "cut"), 1);
    }

    #[test]
    fn kitten_sitting() {
        assert_eq!(edit_distance("kitten", "sitting"), 3);
    }

    #[test]
    fn flaw_lawn() {
        assert_eq!(edit_distance("flaw", "lawn"), 2);
    }

    #[test]
    fn gumbo_gambol() {
        assert_eq!(edit_distance("gumbo", "gambol"), 2);
    }

    #[test]
    fn completely_different() {
        assert_eq!(edit_distance("abc", "xyz"), 3);
    }

    #[test]
    fn repeated_characters() {
        assert_eq!(edit_distance("aaaaa", "aaa"), 2);
        assert_eq!(edit_distance("aaa", "aaaaa"), 2);
    }

    #[test]
    fn transposition() {
        assert_eq!(edit_distance("ab", "ba"), 2);
    }

    #[test]
    fn long_common_prefix() {
        assert_eq!(
            edit_distance(
                "abcdefghijklmnopqrstuvwxyy",
                "abcdefghijklmnopqrstuvwxzz"
            ),
            2
        );
    }

    #[test]
    fn long_common_suffix() {
        assert_eq!(
            edit_distance(
                "zzabcdefghijklmnopqrstuvwxyz",
                "yyabcdefghijklmnopqrstuvwxyz"
            ),
            2
        );
    }

    #[test]
    fn case_sensitive() {
        assert_eq!(edit_distance("Hello", "hello"), 1);
    }

    #[test]
    fn whitespace() {
        assert_eq!(edit_distance("hello world", "helloworld"), 1);
    }

    #[test]
    fn punctuation() {
        assert_eq!(edit_distance("hello!", "hello?"), 1);
    }

    #[test]
    fn unicode_ascii() {
        assert_eq!(edit_distance("café", "cafe"), 1);
    }

    #[test]
    fn emoji_whitespace() {
        assert_eq!(edit_distance("😀😃😄", "😀😄"), 1);
    }

    #[test]
    fn symmetry() {
        let a = "execution";
        let b = "intention";

        assert_eq!(edit_distance(a, b), edit_distance(b, a));
    }

    #[test]
    fn triangle_like_example() {
        let a = "cat";
        let b = "cats";
        let c = "cuts";

        assert!(edit_distance(a, c) <= edit_distance(a, b) + edit_distance(b, c));
    }

    #[test]
    fn exact_match() {
        let result = approximate_match("hello", "hello");

        assert_eq!(result.distance, 0);
        assert_eq!(result.matched_string, "hello");
    }

    #[test]
    fn exact_match_inside_text() {
        let result = approximate_match("hello", "xxx hello yyy");

        assert_eq!(result.distance, 0);
        assert_eq!(result.matched_string, "hello");
    }

    #[test]
    fn one_substitution() {
        let result = approximate_match("hello", "xxx hella yyy");

        assert_eq!(result.distance, 1);
        assert_eq!(result.matched_string, "hella");
    }

    #[test]
    fn one_insertion() {
        let result = approximate_match("hello", "xxx helllo yyy");

        assert_eq!(result.distance, 1);
        assert_eq!(result.matched_string, "helllo");
    }

    #[test]
    fn one_deletion() {
        let result = approximate_match("hello", "xxx helo yyy");

        assert_eq!(result.distance, 1);
        assert_eq!(result.matched_string, "helo");
    }

    #[test]
    fn completely_different_string() {
        let result = approximate_match("abc", "xyz");

        assert_eq!(result.distance, 3);
        assert_eq!(result.matched_string, "xyz");
    }

    #[test]
    fn chooses_best_substring() {
        let result = approximate_match("hello", "this is something hello something else",
        );

        assert_eq!(result.distance, 0);
        assert_eq!(result.matched_string, "hello");
    }

    #[test]
    fn best_match_is_not_entire_text() {
        let result = approximate_match("cat", "xxxxxxxxcatxxxxxxxx",
        );

        assert_eq!(result.distance, 0);
        assert_eq!(result.matched_string, "cat");
    }

    #[test]
    fn approximate_match_result() {
        let result = approximate_match("kitten", "xxx sitten yyy",
        );

        assert_eq!(result.distance, 1);
        assert_eq!(result.matched_string, "sitten");
    }

    #[test]
    fn match_at_start() {
        let result = approximate_match("hello", "hello world",
        );

        assert_eq!(result.start, 0);
        assert_eq!(result.end, 5);
        assert_eq!(result.distance, 0);
        assert_eq!(result.matched_string, "hello");
    }

    #[test]
    fn match_at_end() {
        let result = approximate_match("world", "hello world",
        );

        assert_eq!(result.distance, 0);
        assert_eq!(result.matched_string, "world");
    }

    #[test]
    fn empty_pattern() {
        let result = approximate_match("", "hello");

        assert_eq!(result.distance, 0);
        assert_eq!(result.matched_string, "");
    }

    #[test]
    fn empty_text() {
        let result = approximate_match("hello", "");

        assert_eq!(result.distance, 5);
        assert_eq!(result.matched_string, "");
    }

    #[test]
    fn both_empty() {
        let result = approximate_match("", "");

        assert_eq!(result.distance, 0);
        assert_eq!(result.matched_string, "");
    }

    #[test]
    fn unicode() {
        let result = approximate_match("hello", "xxx héllo yyy",
        );

        assert_eq!(result.distance, 1);
        assert_eq!(result.matched_string, "héllo");
    }

    #[test]
    fn emoji() {
        let result = approximate_match("hello", "😀 hello 😀",
        );

        assert_eq!(result.distance, 0);
        assert_eq!(result.matched_string, "hello");
    }
}
