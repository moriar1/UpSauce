use anyhow::{Context, Error, Result};
use rustnao::HandlerBuilder;

pub struct Sauce {
    pub name: String,
    pub url: String,
}

/// Returns Markdown String of sauces splitted with `delim`:
/// [sauce1](http..) | [sauce2](http..) | [sauce3](http..)
///
/// # Errors
///
/// see `get_sauces()`
pub async fn get_pretty_sauces(
    image_path: &str,
    image_fss_path: &str, // Might be empty
    delim: &str,
    api_key: &str,
) -> Result<String> {
    let (sauces, skipped) = get_sauces(image_path, api_key).await?;
    let mut md_string = String::new();

    for sauce in sauces {
        md_string += &format!("[{}]({}){}", sauce.name, sauce.url, delim);
    }
    for skipped in &skipped {
        println!("Skipped ext_url: {skipped}");
    }

    if !skipped.is_empty() {
        println!();
    }

    if image_fss_path.is_empty() {
        md_string.pop(); // remove last `delim`
        Ok(md_string)
    } else {
        Ok(md_string + &format!("[Image]({})", image_fss_path))
    }
}
#[must_use]
pub fn get_sauce_name(source_url: &str) -> &'static str {
    //TODO: Maybe better use lazy_static
    let sources: &[(&str, &str)] = &[
        ("gelbooru", "Gelbooru"),
        ("danbooru", "Danbooru"),
        ("yande.re", "yande.re"),
        ("konachan.", "Konachan"),
        ("deviantart", "DeviantArt"),
        ("twitter.com", "Twitter"),
        ("pixiv", "Pixiv"),
        ("anime-pictures.net", "anime-pictures.net"),
        ("artstation.com", "ArtStation"),
        ("tumblr.com", "Tumblr"),
        ("https:://x.com", "Twitter"),
        ("instagram.com", "Instagram"),
        ("misskey", "Misskey"),
        ("bsky", "Bluesky"),
    ];

    if source_url.is_empty() {
        return "";
    }
    let mut src = "";
    for &(keyword, source_name) in sources {
        if source_url.contains(keyword) {
            src = source_name;
            break;
        }
    }
    src
}

/// Returns `Vec` with known sauces (pixiv, gelbooru...) and `Vec` with unknown or garbage sauces' url (sankaku, nico...)
/// ## Arguments
/// * ``image_path`` - A string slice that contains the url of the image you wish to look up.
/// * ``api_key`` - A string reference representing your API key.
///
/// ## Errors
/// From `RustNAO`:
///
/// > If there was a problem forming a URL, reading a file, making a request, or parsing the returned JSON, an error will be returned.
/// > Furthermore, if you pass a link in which SauceNAO returns an error code, an error containing the code and message will be returned.
pub async fn get_sauces(image_path: &str, api_key: &str) -> Result<(Vec<Sauce>, Vec<String>)> {
    // SauceNAO request
    let handle = HandlerBuilder::default()
        .api_key(api_key)
        .num_results(999)
        .build();
    let result = handle
        .async_get_sauce_as_json(image_path, None, Some(61.0))
        .await
        .map_err(Error::msg)?;
    let json_result: serde_json::Value =
        serde_json::from_str(&result).context("Failed parse SNAO JSON: {e}")?;

    // Creating `Vec` of sauces and Vec of unknown/garbage sauces
    let (mut sauces, mut skipped) = (vec![], vec![]);
    let mut copy_helper = String::new(); // helps to check has sauce already been added

    for snao_obj in json_result.as_array().unwrap_or(&vec![]) {
        if let Some(urls) = snao_obj["ext_urls"].as_array() {
            for url in urls {
                if let Some(url) = url.as_str() {
                    let name = get_sauce_name(url);
                    if name.is_empty() || copy_helper.contains(name)
                    // || sauces.contains(&Sauce {
                    //     name: name.to_owned(),
                    //     url: url.to_owned(),
                    // })
                    {
                        skipped.push(url.to_owned());
                    } else {
                        sauces.push(Sauce {
                            name: name.to_owned(),
                            url: url.to_owned(),
                        });
                        copy_helper.push_str(name);
                    }
                }
            }
        }
        // Sometimes booru object contains `source` object (usually Twitter or Pixiv)
        // But `Sankaku Channel` usually contains garbage, so skip it
        if !snao_obj["site"].as_str().unwrap_or("").contains("Sankaku ") {
            if let Some(url) = snao_obj["additional_fields"]["source"].as_str() {
                let name = get_sauce_name(url);
                if name.is_empty() || copy_helper.contains(name) {
                    skipped.push(url.to_owned());
                } else {
                    sauces.push(Sauce {
                        name: name.to_owned(),
                        url: url.to_owned(),
                    });
                    copy_helper.push_str(name);
                }
            }
        }
    }

    Ok((sauces, skipped))
}
