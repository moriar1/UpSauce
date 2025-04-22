use anyhow::{bail, Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::{fs::File, io::BufReader, path::PathBuf};
use upsauce::{sauce::get_sauces, MyClient};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let cfg = Config::read_config("config.json", &cli)?;

    if cli.skip_upload {
        if !cli.path.starts_with("https://") {
            bail!("Provide URL if using -s, --skip-upload")
        }
        print_sauces(&cli.path, "", &cfg.delim, &cfg.api_key).await?;
        return Ok(());
    }

    let client = MyClient::new();
    // Get path to image from command line argument or download it if url is provided
    let path = if cli.path.starts_with("https://") {
        println!(":: Dowloading image from {}", &cli.path);
        client.download_image(&cli.path).await?
    } else if !PathBuf::from(&cli.path).exists() {
        bail!("File do not exists.")
    } else {
        cli.path
    };

    if cli.catbox {
        println!(":: Uploading image on catbox.moe");
        let fss_image_url = upsauce::MyClient::catbox_upload(&path, None).await?;
        println!(":: Getting sauce");
        print_sauces(&fss_image_url, &fss_image_url, &cfg.delim, &cfg.api_key).await?;
    } else {
        let linx_url = cfg
            .linx_url
            .context("Neither CLI nor config.json contain linx-server URL")?;

        println!(":: Uploading image on linx-server: {}", linx_url);
        let l = client.linx_upload(&path, &linx_url).await?;
        println!(
            "-> To delete your file on `{}` use: `curl -H \"Linx-Delete-Key: {}\" -X DELETE {}`",
            linx_url, l.delete_key, l.url
        );

        println!(":: Getting sauce");
        print_sauces(&l.direct_url, &l.url, &cfg.delim, &cfg.api_key).await?;
        println!("{}", l.direct_url);
    }

    Ok(())
}

/// Returns Markdown String of sauces splitted with `delim`:
/// [sauce1](http..) | [sauce2](http..) | [sauce3](http..)
/// and `Vec` of unknown sauces.
///
/// # Errors
///
/// see `get_sauces()`
async fn print_sauces(
    image_path: &str,     // URL or local path
    image_fss_path: &str, // Additional link at the end of MD String, might be empty
    delim: &str,          // delimiter between every sauce
    api_key: &str,        // SauceNAO API key
) -> Result<()> {
    let (sauces, skipped_sauces) = get_sauces(image_path, api_key).await?;
    let mut md_string = String::new();

    for sauce in sauces {
        md_string += &format!("[{}]({}){}", sauce.name, sauce.url, delim);
    }

    let md_string = if image_fss_path.is_empty() {
        md_string[0..md_string.len() - delim.len()].to_string() // delete last delim
    } else {
        md_string + &format!("[Image]({})", image_fss_path)
    };
    println!("{}", md_string);

    if !skipped_sauces.is_empty() {
        println!();
        println!("Skipped ext_url:");
        for i in skipped_sauces {
            println!("{}", i);
        }
    }
    Ok(())
}

#[derive(Deserialize, Serialize, Debug)]
struct Config {
    api_key: String,

    #[serde(default = "default_delimiter")]
    delim: String,

    #[serde(default)]
    linx_url: Option<String>,
}

fn default_delimiter() -> String {
    " | ".to_owned()
}

impl Config {
    fn read_config(config_path: &str, cli: &Cli) -> Result<Config> {
        if cli.api_key.is_empty() {
            // Try read API key from config.json
            match File::open(config_path) {
                // Read all data from config.json
                Ok(file) => Ok(serde_json::from_reader(BufReader::new(file))
                    .context("Failed reading `config.json`")?),
                Err(_) => None.context("Neither CLI nor config.json contain SauceNAO API key"),
            }
        } else {
            // If API key found in CLI args than get other parameters from CLI
            let delim = if cli.delim.is_empty() {
                " | ".to_owned()
            } else {
                cli.delim.clone()
            };
            Ok(Config {
                api_key: cli.api_key.clone(),
                delim,
                linx_url: cli.linx_url.clone(),
            })
        }
    }
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Local path or URL to image
    path: String,

    /// SauceNAO API key
    #[arg(short, long, default_value_t)]
    api_key: String,

    /// Source delimiter, " | " by default, use quotes
    #[arg(short, long, default_value_t)]
    delim: String,

    /// Linx-server URL, for exapmle https://put.icu
    #[arg(short, long)]
    linx_url: Option<String>,

    /// Skip uploading file on FSS (url only)
    #[arg(short, long)]
    skip_upload: bool,

    /// Upload on catbox.moe instead of linx-server instance
    #[arg(short, long)]
    catbox: bool,
}
