use std::{
    env::args,
    process::{Command, Stdio},
};

use rustnao::HandlerBuilder;

fn upload_cdn(path: &str) -> (String, String, String) {
    let a = [
        "--http1.1",
        "-H",
        "Accept: application/json",
        "-T",
        path,
        "https://shota.nu/upload/",
    ];

    let output = Command::new("curl")
        .args(a)
        .stderr(Stdio::inherit())
        .output()
        .unwrap();

    let json_result: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap();

    //# For debug
    // let json_str = std::fs::read_to_string("./output.json").expect("No json");
    // let json_result: serde_json::Value =
    //     serde_json::from_str(&json_str).expect("result JSON not well f");

    let direct_url = json_result["direct_url"].as_str().unwrap();
    let url = json_result["url"].as_str().unwrap();
    let delete_key = json_result["delete_key"].as_str().unwrap();
    (direct_url.to_owned(), url.to_owned(), delete_key.to_owned())
}

fn get_pretty_sauce(url: &str) -> Result<String, String> {
    let data =
        std::fs::read_to_string("config.json").map_err(|e| format!("Couldn't read file: {e}"))?;

    // Get SauceNAO API key
    let json: serde_json::Value = serde_json::from_str(data.as_str())
        .map_err(|e| format!("JSON not well formatted: {e}."))?;
    let key = json["api_key"]
        .as_str()
        .ok_or_else(|| "No api key".to_string())?;

    // SauceNAO request
    let handle = HandlerBuilder::default()
        .api_key(key)
        .num_results(999)
        .build();
    handle.set_min_similarity(61.0);
    let result = handle
        .get_sauce_as_pretty_json(url, None, None)
        .map_err(|e| format!("Cannot get sauce: {e}"))?;
    let json_result: serde_json::Value =
        serde_json::from_str(&result).map_err(|e| format!("Failed parse SNAO JSON: {e}"))?;

    //# For debug
    // let data = std::fs::read_to_string("./output.json").expect("No json");
    // let json_result: serde_json::Value =
    //     serde_json::from_str(&data).expect("result JSON not well f");

    // Make pretty string with sauce
    let sauce = json_result
        .as_array()
        .unwrap() // SNAO JSON always contains array
        .iter()
        .fold(String::new(), |acc, entry| {
            acc + &format!(
                "[{}]({})・",
                entry["site"].as_str().unwrap(),
                entry["ext_urls"][0].as_str().unwrap() // TODO: may be more than 1 ext_urls!
            )
        });
    Ok(sauce + &format!("[CDN]({})", url.replace("/ss", "")))
}

fn check_yes_input() -> bool {
    let mut input = String::new();
    if std::io::stdin().read_line(&mut input).is_err() {
        false
    } else {
        input.trim().to_lowercase() == "y"
    }
}

fn main() {
    // TODO: Provide link or local path to image
    // let path = download_img(url); or path = `local`;
    let path = args().nth(1).expect("err: provide path/to/image");

    // Upload image to shota.nu
    let (direct_url, url, delete_key) = upload_cdn(&path);

    // Get Sauce from SauceNAO
    match get_pretty_sauce(&direct_url) {
        Ok(pretty_sauce) => println!("\n{pretty_sauce}\n{direct_url}\n"), //"),
        Err(e) => {
            println!("Failed sauce fetching. {e}\nDelete image uploaded on `shota.nu`? [y/n]");
            if check_yes_input() {
                if let Err(e) = delete_image_request(&url) {
                    println!("{e}");
                } else {
                    println!("Deleted succesfully");
                }
            }
        }
    }
    println!("To delete your file on shota.nu use: `curl -H \"Linx-Delete-Key: {delete_key}\" -X DELETE {url}`");

    // Upload on
}

fn delete_image_request(url: &str) -> Result<(), String> {
    Err("Deletion failed".to_owned())
}
