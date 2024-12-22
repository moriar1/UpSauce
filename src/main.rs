use core::panic;
use rustnao::HandlerBuilder;
use std::{
    env::args,
    process::{Command, Stdio},
};
use url::Url;
// Since UpSauce uses curl via `std::prcoess::Command`- using `url::Url` adds
// unecessary convertations, so `&str` is preferable (the same for `PathBuf`)

fn download_image(image_url: &str) -> Result<String, String> {
    Command::new("curl")
        .args(["-O", image_url])
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .output()
        .map_err(|e| format!("Failed curl request: {e}"))?;

    let binding = Url::parse(image_url).unwrap();
    let image_name = binding.path_segments().unwrap().last().unwrap();
    Ok(image_name.to_string())
}

fn upload(path: &str, linx_url: &str) -> Result<(String, String, String), String> {
    let url_upload = linx_url.to_owned() + "/upload/";
    let curl_stdout = Command::new("curl")
        .args(["-H", "Accept: application/json", "-T", path, &url_upload])
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| format!("Failed curl request: {e}"))?
        .stdout;
    // TODO: might be better utilizing tokio or reqwest

    let curl_stdout = String::from_utf8_lossy(&curl_stdout);

    let json_result: serde_json::Value = serde_json::from_str(&curl_stdout)
        .map_err(|e| format!("Bad linx-server response: {e}. Curl stdout:\n{curl_stdout}"))?;

    let direct_url = json_result["direct_url"]
        .as_str()
        .ok_or("No direct_url from linx-server")?;
    let url = json_result["url"].as_str().ok_or("No url from linx")?;
    let delete_key = json_result["delete_key"]
        .as_str()
        .ok_or("No delete key from linx-server")?;
    Ok((direct_url.to_owned(), url.to_owned(), delete_key.to_owned()))
}

fn get_pretty_sauce(direct_url: &str, linx_file_url: &str, delim: &str) -> Result<String, String> {
    // Get SauceNAO API key
    let data = std::fs::read_to_string("config.json")
        .map_err(|e| format!("Failed reading `config.json`: {e}"))?;
    let json: serde_json::Value = serde_json::from_str(data.as_str())
        .map_err(|e| format!("JSON not well formatted: {e}."))?;
    let key = json["api_key"].as_str().ok_or("No api key".to_string())?;

    // SauceNAO request
    let handle = HandlerBuilder::default()
        .api_key(key)
        .num_results(999)
        .build();
    handle.set_min_similarity(61.0);
    let result = handle
        .get_sauce_as_pretty_json(direct_url, None, None)
        .map_err(|e| format!("Cannot get sauce: {e}"))?;
    let json_result: serde_json::Value =
        serde_json::from_str(&result).map_err(|e| format!("Failed parse SNAO JSON: {e}"))?;

    // Creating pretty sauce string based on SauceNAO json response
    let sources = [
        ("gelbooru", "Gelbooru"),
        ("danbooru", "Danbooru"),
        ("yande.re", "Yande.re"),
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
                    eprintln!("Skipped ext_url: {url}");
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

fn check_yes_input() -> bool {
    let mut input = String::new();
    if std::io::stdin().read_line(&mut input).is_err() {
        false
    } else {
        input.trim().to_lowercase() == "y"
    }
}

fn delete_image_request(url: &str, key: &str) -> Result<(), String> {
    let del_key_str = format!("Linx-Delete-Key: {key}");
    let a = ["curl", "-H", del_key_str.as_str(), "-X", "DELETE", url];

    let output = Command::new("curl")
        .args(a)
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| format!("Failed curl request: {e}"))?;

    let response = String::from_utf8_lossy(&output.stdout);

    if response == "DELETED" {
        Ok(())
    } else {
        Err("Response: \n".to_owned() + &response)
    }
}

fn main() {
    // Get path to image from command line argument or download it if url is provided
    let path = args().nth(1).expect("error: provide path/to/image"); // Local path or url to image
    let path = if path.starts_with("https://") {
        println!("Dowloading image from {path}");
        download_image(&path).unwrap_or_else(|e| panic!("Failed downloading provided image: {e}"))
    } else {
        path
    };

    let delim = " | "; // Delimiter between sources in Markdown string
    let linx_url = "https://put.icu"; // linx-server instance url

    println!("Uploading image to {linx_url}");
    let (direct_url, url, delete_key) = upload(&path, linx_url)
        .unwrap_or_else(|e| panic!("Failed uploading your image on linx-server: {e}"));
    // TODO: add visual feedback

    println!("Searching for sauce");
    // Get sauce from SauceNAO
    match get_pretty_sauce(&direct_url, &url, delim) {
        Ok(pretty_sauce) => println!("\n{pretty_sauce}\n{direct_url}\n"),
        Err(e) => {
            println!("Failed sauce fetching. {e}\nDelete image uploaded on `{linx_url}`? [y/n]");
            if check_yes_input() {
                if let Err(e) = delete_image_request(&url, &delete_key) {
                    println!("Deletion failed. {e}");
                } else {
                    println!("Deleted succesfully");
                    return; // skip printing delete key
                }
            }
        }
    }
    println!("To delete your file on `{linx_url}` use: `curl -H \"Linx-Delete-Key: {delete_key}\" -X DELETE {url}`");
}
