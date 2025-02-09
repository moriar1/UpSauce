use anyhow::{bail, Context, Result};
use reqwest::{header::HeaderValue, Client};
use std::process::Stdio;
use tokio::{
    fs::{self, File},
    io::{AsyncReadExt, AsyncWriteExt},
    process::Command,
};
use url::Url;

pub struct LinxResponse {
    pub url: String,
    pub direct_url: String,
    pub delete_key: String,
}

impl MyClient {
    pub async fn download_image(&self, image_url: &str) -> Result<String> {
        let binding = Url::parse(image_url).context("Incorrect url.")?;
        let image_name = binding.path_segments().context("")?.last().context("")?;

        let res = self.get(image_url).send().await.context("Fail downl img")?;
        let content_type = res
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("");

        if !content_type.starts_with("image/") {
            bail!(format!("Response is not an image, but {content_type}"));
        }

        let content = res.bytes().await.context("Failed download image.")?;
        fs::File::create(image_name)
            .await
            .context("Failed creating file to image")?
            .write_all(&content)
            .await
            .context("Failed downloading image")?;
        Ok(image_name.to_string())
    }

    pub async fn linx_upload(&self, path: &str, linx_url: &str) -> Result<LinxResponse> {
        let url_upload = linx_url.to_owned() + "/upload/";

        let mut file = File::open(path)
            .await
            .with_context(|| format!("Failed open `{path}`"))?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)
            .await
            .with_context(|| format!("Failed read `{path}`"))?;

        let res = self
            .put(url_upload)
            .header("Accept", HeaderValue::from_static("application/json"))
            .body(data)
            .send()
            .await
            .context("Failed to send request to linx server")?
            .text()
            .await
            .context("Failed to read text response from linx server")?;

        let json_result: serde_json::Value = serde_json::from_str(&res)
            .with_context(|| format!("Bad linx-server response:\n{res}"))?;

        let extract_string = |key: &str| -> Result<String> {
            json_result[key]
                .as_str()
                .with_context(|| format!("No {key} from linx"))
                .map(std::borrow::ToOwned::to_owned)
        };
        Ok(LinxResponse {
            url: extract_string("url")?,
            direct_url: extract_string("direct_url")?,
            delete_key: extract_string("delete_key")?,
        })
    }

    // pub fn delete_image_request(url: &str, key: &str) -> Result<()> {
    //     let response = reqwest::blocking::Client::new()
    //         .delete(url)
    //         .header("Linx-Delete-Key", key)
    //         .send()
    //         .context("Failed deleting file.")?
    //         .text()
    //         .context("Failed deleting file.")?;

    //     if response == "DELETED" {
    //         Ok(())
    //     } else {
    //         Err(anyhow!("Response: \n{response}"))
    //     }
    // }

    // calls `curl -F "reqtype=fileupload" -F "userhash=####" -F "fileToUpload=@image.jpg" https://catbox.moe/user/api.php`
    pub async fn catbox_upload(path: &str, _userhash: Option<&str>) -> Result<String> {
        let f = &format!("fileToUpload=@{path}");
        let args = vec![
            "-F",
            "reqtype=fileupload",
            "-F",
            f,
            "https://catbox.moe/user/api.php",
        ];

        // if let Some(hash) = userhash {
        //     let userhash_arg = format!("userhash={}", hash); // Store in a variable
        //     args.push("-F");
        //     args.push(&userhash_arg); // Use the variable here
        // }

        let output = Command::new("curl")
            .args(args)
            .stderr(Stdio::inherit())
            .output()
            .await
            .context("Failed curl request.")?;
        if !output.status.success() {
            None.with_context(|| format!("Failed upload {path}"))?;
        }

        let response = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(response)
    }
}

pub struct MyClient {
    pub client: Client,
}

impl std::ops::Deref for MyClient {
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl Default for MyClient {
    fn default() -> Self {
        Self::new()
    }
}

impl MyClient {
    #[must_use]
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}
