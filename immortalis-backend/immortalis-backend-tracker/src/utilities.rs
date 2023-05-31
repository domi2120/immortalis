pub fn is_youtube_video_collection(url: &str) -> bool {
    url.ends_with("videos")
        || url.ends_with("streams")
        || url.ends_with("shorts")
        || url.ends_with("videos/")
        || url.ends_with("streams/")
        || url.ends_with("shorts/")
}
