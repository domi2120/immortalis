use std::ops::Deref;

/// Receives an URL as &str and returns it with only query parameters with names contained in accepted_parameter_names
/// ```rust
/// assert_eq(filter_query_pairs("https://example.net/?lang=en&foo=bar".toString(), vec!["lang"]), "https://example.net/?lang=en")
/// ```
pub fn filter_query_pairs(initial_url: &str, accepted_parameter_names: Vec<&str>) -> String {
    // trim query params other than 'v' which is the video (trims for example playlists)
    let url = url::Url::parse(initial_url).unwrap();
    let view_query_param = url
        .query_pairs()
        .filter(|x| accepted_parameter_names.contains(&x.0.deref()));
    let mut new_url = url.clone();
    new_url
        .query_pairs_mut()
        .clear()
        .extend_pairs(view_query_param);
    new_url.to_string()
}
