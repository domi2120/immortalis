#[derive(PartialEq, Eq, Debug)]
pub enum UrlType {
    Invalid,
    Video,
    Collection,
    VideoOrCollection
}

pub fn get_url_type(url: &str) -> UrlType {

    if url.is_empty() || !url.contains("youtube.com") {
        return UrlType::Invalid
    }

    if url.ends_with("videos")
        || url.ends_with("streams")
        || url.ends_with("shorts")
        || url.ends_with("videos")
        || url.ends_with("streams")
        || url.ends_with("shorts")
        || url.ends_with("podcasts")
        || url.ends_with("playlists")
        || url.ends_with("releases")
        || url.contains("/@")
        || url.contains("/channel") {
        UrlType::Collection
    } else {
        if url.contains("list=") && url.contains("v=") {
            UrlType::VideoOrCollection
        } else if url.contains("list=") {
            UrlType::Collection
        } else if url.contains("v=") || url.contains("shorts/") {
            UrlType::Video
        } else {
            UrlType::Invalid
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::utilities::UrlType;

    use super::get_url_type;

    #[test]
    fn test_get_url_type() {
        assert_eq!(get_url_type(""), UrlType::Invalid);
        assert_eq!(get_url_type("abc"), UrlType::Invalid);
        assert_eq!(get_url_type("https://crates.io"), UrlType::Invalid);
        assert_eq!(get_url_type("https://www.youtube.com"), UrlType::Invalid);
        assert_eq!(get_url_type("https://www.youtube.com/channel/testChannel"), UrlType::Collection);
        assert_eq!(get_url_type("https://www.youtube.com/@test"), UrlType::Collection);
        assert_eq!(get_url_type("https://www.youtube.com/@test/shorts"), UrlType::Collection);
        assert_eq!(get_url_type("https://www.youtube.com/shorts/anotherTest"), UrlType::Video);
        assert_eq!(get_url_type("https://www.youtube.com/watch?v=testVideoId"), UrlType::Video);
        assert_eq!(get_url_type("https://www.youtube.com/watch?v=testVideoId&list=playListId"), UrlType::VideoOrCollection);
        assert_eq!(get_url_type("https://www.youtube.com/playlist?list=playListId"), UrlType::Collection);
        
    }
}