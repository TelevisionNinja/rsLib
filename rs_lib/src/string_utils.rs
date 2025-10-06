use std::collections::HashSet;

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
            let mut result = String::new();
            let url_string = url.to_string();

            let mut parameters = super::HashSet::new();
            parameters.insert("si".to_string());
            parameters.insert("pp".to_string());

            //---------------------

            let mut first_chunk_iterator = url_string.split("?");
            let first_chunk = first_chunk_iterator.next().unwrap();
            let second_chunk = first_chunk_iterator.next();

            if !second_chunk.is_some() {
                return url.to_string();
            }

            result.push_str(first_chunk);
            let second_chunk_unwrapped = second_chunk.unwrap();

            //---------------------

            let parameter_iterator = second_chunk_unwrapped.split("&");
            let filtered_parameters = parameter_iterator.filter(|parameter| !parameters.contains(parameter.split("=").next().unwrap()));
            let remaining_parameters = filtered_parameters.collect::<Vec<&str>>().join("&");

            if !remaining_parameters.is_empty() {
                result.push_str(&("?".to_owned() + &remaining_parameters));
            }

            return result.to_string();
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
}
