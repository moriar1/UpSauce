use anyhow::{bail, Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::{fs::File, io::BufReader, path::PathBuf};
use upsauce::{sauce::get_pretty_sauces, MyClient};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let cfg = Config::read_config("config.json")?;

    if cli.skip_upload {
        if !cli.path.starts_with("https://") {
            bail!("Provide URL if using -s, --skip-upload")
        }
        let s = get_pretty_sauces(&cli.path, "", &cfg.delim, &cfg.api_key).await?;
        println!("{s}");
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
        let s = get_pretty_sauces(&fss_image_url, &fss_image_url, &cfg.delim, &cfg.api_key).await?;
        println!("{s}");
    } else {
        println!(":: Uploading image on linx-server: {}", cfg.linx_url);
        let l = client.linx_upload(&path, &cfg.linx_url).await?;
        println!(
            ":: To delete your file on `{}` use: `curl -H \"Linx-Delete-Key: {}\" -X DELETE {}`",
            cfg.linx_url, l.delete_key, l.url
        );
        println!(":: Getting sauce");
        let s = get_pretty_sauces(&l.direct_url, &l.url, &cfg.delim, &cfg.api_key).await?;
        println!("{}\n{}", s, l.direct_url);
    }

    Ok(())
}

#[derive(Deserialize, Serialize, Debug)]
struct Config {
    api_key: String,
    delim: String,

    #[serde(default)]
    linx_url: String,
}

impl Config {
    fn read_config(path: &str) -> Result<Config> {
        let rdr = BufReader::new(File::open(path).context("Failed to read `config.json`")?);
        let config: Config =
            serde_json::from_reader(rdr).context("Failed reading `config.json`")?;
        Ok(config)
    }
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    path: String,

    /// Skip uploading file on FSS (url only)
    #[arg(short, long)]
    skip_upload: bool,

    /// Upload on catbox.moe instead of linx-server instance
    #[arg(short, long)]
    catbox: bool,
}
