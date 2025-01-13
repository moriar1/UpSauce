use rustnao::HandlerBuilder;

pub fn get_pretty_sauce(
    linx_direct_url: &str,
    linx_file_url: &str,
    delim: &str,
    api_key: &str,
) -> Result<String, String> {
    // SauceNAO request
    let handle = HandlerBuilder::default()
        .api_key(api_key)
        .num_results(999)
        .build();
    handle.set_min_similarity(61.0);
    let result = handle
        .get_sauce_as_pretty_json(linx_direct_url, None, None)
        .map_err(|e| format!("Cannot get sauce: {e}"))?;
    let json_result: serde_json::Value =
        serde_json::from_str(&result).map_err(|e| format!("Failed parse SNAO JSON: {e}"))?;

    // Creating pretty sauce string based on SauceNAO json response
    let sources = [
        ("gelbooru", "Gelbooru"),
        ("danbooru", "Danbooru"),
        ("yande.re", "yande.re"),
        ("konachan.", "Konachan"),
        ("deviantart", "DeviantArt"),
        ("twitter.com", "Twitter"),
        ("pixiv.net", "Pixiv"),
        ("anime-pictures.net", "anime-pictures.net"), // Not tested
        ("artstation.com", "ArtStation"),             // Not tested
        ("tumblr.com", "Tumblr"),                     // Not tested
        ("https:://x.com", "Twitter"),                // Not tested
        ("instagram.com", "Instagram"),               // Not tested
        ("misskey", "Misskey"),                       // Not tested
        ("bsky", "Bluesky"),                          // Not tested
    ];
    let mut sauce = String::new();

    let mut flag_skipped_source = true;
    for snao_obj in json_result.as_array().unwrap_or(&vec![]) {
        if let Some(urls) = snao_obj["ext_urls"].as_array() {
            for url in urls {
                if let Some(url) = url.as_str() {
                    for &(keyword, source_name) in &sources {
                        if url.contains(keyword) && !sauce.contains(source_name) {
                            sauce += &format!("[{source_name}]({url}){delim}");
                            flag_skipped_source = false;
                            break;
                        }
                    }
                }
                if flag_skipped_source {
                    eprintln!("Skipped ext_url: {}", url.as_str().unwrap_or(""));
                }
                flag_skipped_source = true;
            }
        }
        // Sometimes booru object contains `source` object (usually Twitter or Pixiv)
        // But `Sankaku Channel` usually contains garbage, so skip it
        if !snao_obj["site"].as_str().unwrap_or("").contains("Sankaku ") {
            if let Some(source_url) = snao_obj["additional_fields"]["source"].as_str() {
                // For loop as above (might be better implement separate function)
                for &(keyword, source_name) in &sources {
                    if source_url.contains(keyword) && !sauce.contains(source_name) {
                        sauce += &format!("[{source_name}]({source_url}){delim}");
                        break;
                    }
                }
            }
        }
    }

    Ok(sauce + &format!("[Image]({})", linx_file_url))
}
