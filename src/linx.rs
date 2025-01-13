use std::process::{Command, Stdio};

use url::Url;
// Since UpSauce uses curl via `std::prcoess::Command`- using `url::Url` adds
// unecessary convertations, so `&str` is preferable (the same for `PathBuf`)

pub fn download_image(image_url: &str) -> Result<String, String> {
    Command::new("curl")
        .args(["-O", image_url])
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .output()
        .map_err(|e| format!("Failed curl request: {e}"))?; // TODO: check return value

    let binding = Url::parse(image_url).unwrap();
    let image_name = binding.path_segments().unwrap().last().unwrap();
    Ok(image_name.to_string())
}

pub fn upload(path: &str, linx_url: &str) -> Result<(String, String, String), String> {
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

pub fn check_yes_input() -> bool {
    let mut input = String::new();
    if std::io::stdin().read_line(&mut input).is_err() {
        false
    } else {
        input.trim().to_lowercase() == "y"
    }
}

pub fn delete_image_request(url: &str, key: &str) -> Result<(), String> {
    let del_key_str = format!("Linx-Delete-Key: {key}");
    let a = ["-H", del_key_str.as_str(), "-X", "DELETE", url];

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
