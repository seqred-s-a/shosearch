extern crate reqwest;
extern crate url;

use url::Url;

pub fn api_info(api_key: &String) -> Result<String, Box<std::error::Error>> {
    let url = Url::parse_with_params("https://api.shodan.io/api-info", &[("key", api_key)])?;
    let response = reqwest::get(url)?.text()?;
    Ok(response)
}

pub fn host_search(api_key: &String, query: &String) -> Result<String, Box<std::error::Error>> {
    let url = Url::parse_with_params(
        "https://api.shodan.io/shodan/host/search",
        &[
            ("key", api_key),
            ("minify", &"false".to_string()),
            ("query", query),
        ],
    )?;
    let response = reqwest::get(url)?.text()?;
    Ok(response)
}

pub fn host_search_paged(
    api_key: &String,
    query: &String,
    page: usize,
) -> Result<String, Box<std::error::Error>> {
    let url = Url::parse_with_params(
        "https://api.shodan.io/shodan/host/search",
        &[
            ("key", api_key),
            ("minify", &"false".to_string()),
            ("query", query),
            ("page", &page.to_string()),
        ],
    )?;
    let response = reqwest::get(url)?.text()?;
    Ok(response)
}
