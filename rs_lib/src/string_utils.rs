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
            let mut result = url.to_string();

            let parameters = [
                "si",
                "pp"
            ];

            for parameter in parameters {
                let start_index_result = result.find(&(parameter.to_owned() + "="));

                if start_index_result.is_some() {
                    let start_index = start_index_result.unwrap();
                    // the '&' infront of 'si=' is assumed. thus we get rid of the '&' at the end
                    // another case: the '?' is infront of 'si='. thus we get rid of the '&' at the end
                    let end_index = super::find(&result, "&", start_index).map_or(result.len(), |i| i + 1);

                    result.replace_range(start_index..end_index, "");
                }
            }

            return result;
        }

        url.to_string()
    }
}

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
        assert_eq!("https://youtube.com/shorts/60gZOXu5gcQ?", youtube::remove_tracking_parameters("https://youtube.com/shorts/60gZOXu5gcQ?si=EkAu2o2eUgp4SZV-"));
        assert_eq!("https://www.youtube.com/shorts/H3O6-SHr2fc", youtube::remove_tracking_parameters("https://www.youtube.com/shorts/H3O6-SHr2fc"));
        assert_eq!("https://www.youtube.com/watch?v=HCE_lFUMXNg", youtube::remove_tracking_parameters("https://www.youtube.com/watch?si=EkAu2o2eUgp4SZV-&v=HCE_lFUMXNg"));

        assert_eq!("https://youtube.com/shorts/60gZOXu5gcQ?", youtube::remove_tracking_parameters("https://youtube.com/shorts/60gZOXu5gcQ?pp=EkAu2o2eUgp4SZV-"));
        assert_eq!("https://www.youtube.com/watch?v=HCE_lFUMXNg", youtube::remove_tracking_parameters("https://www.youtube.com/watch?pp=EkAu2o2eUgp4SZV-&v=HCE_lFUMXNg"));

        assert_eq!("https://www.youtube.com/watch?v=HCE_lFUMXNg&", youtube::remove_tracking_parameters("https://www.youtube.com/watch?si=EkAu2o2eUgp4SZV-&v=HCE_lFUMXNg&pp=EkAu2o2eUgp4SZV-"));
        assert_eq!("https://www.youtube.com/watch?v=HCE_lFUMXNg&", youtube::remove_tracking_parameters("https://www.youtube.com/watch?v=HCE_lFUMXNg&pp=EkAu2o2eUgp4SZV-&si=EkAu2o2eUgp4SZV-"));
    }
}
