use std::{env::args, path::PathBuf};
use upsauce::{check_yes_input, delete_image_request, download_image, get_pretty_sauce, upload};

fn main() {
    const USAGE_STR: &str = "Usage: upsauce <PATH/URL>

Options:
  -h, --help     Print help
  -V, --version  Print version
";

    let arg = args().nth(1).unwrap_or_else(|| {
        println!("{USAGE_STR}");
        String::new()
    });
    let path = match arg.as_str() {
        "" => return,
        "--version" | "-V" => {
            println!("{}", env!("CARGO_PKG_VERSION"));
            return;
        }
        "--help" | "-h" => {
            println!("{USAGE_STR}");
            return;
        }
        _ => arg,
    };

    // Get SauceNAO API key, linx_url, delimiter between sources from `config.json`
    let data = std::fs::read_to_string("config.json")
        .unwrap_or_else(|e| panic!("Failed reading `config.json`: {e}"));
    let json: serde_json::Value = serde_json::from_str(data.as_str())
        .unwrap_or_else(|e| panic!("JSON not well formatted: {e}."));
    let api_key = json["api_key"]
        .as_str()
        .unwrap_or_else(|| panic!("No SaunceNAO API key in `config.json`"));
    let linx_url = json["linx_url"]
        .as_str()
        .unwrap_or_else(|| panic!("No linx_url in `config.json`"));
    let delim = json["delim"].as_str().unwrap_or(" | "); // Delimiter between sources in Markdown string

    // Get path to image from command line argument or download it if url is provided
    let path = if path.starts_with("https://") {
        println!("Dowloading image from {path}");
        download_image(&path).unwrap_or_else(|e| panic!("Failed downloading provided image: {e}"))
    } else if !PathBuf::from(&path).exists() {
        panic!("Path to file do not exists")
    } else {
        path
    };

    println!("Uploading image to {linx_url}");
    let (direct_url, url, delete_key) = upload(&path, linx_url)
        .unwrap_or_else(|e| panic!("Failed uploading your image on linx-server: {e}"));

    println!("Searching for sauce");
    // Get sauce from SauceNAO
    match get_pretty_sauce(&direct_url, &url, delim, api_key) {
        Ok(pretty_sauce) => println!("\n{pretty_sauce}\n{direct_url}\n"),
        Err(e) => {
            println!("Failed sauce fetching. {e}\nDelete image uploaded on `{linx_url}`? [Y/n]");
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
