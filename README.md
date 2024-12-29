# UpSauce

UpSauce uploads your images on [linx-server](https://github.com/ZizzyDizzyMC/linx-server/) instance and
outputs links found on [SauceNAO](https://saucenao.com) as a Markdown string, formatted like this: [Pixiv](https://www.pixiv.net/member_illust.php?mode=medium&illust_id=61477678) | [Twitter](https://twitter.com/i/web/status/837653407900934145) | ... | [Image](https://put.icu/x2zj493c.jpeg)

# Usage
Sign up on [SauceNAO](https://saucenao.com) to obtain your API key. Enter the linx-server instance URL, your SauceNAO API key, and optionally the source delimiter (which defaults to ` | `) in `config.json`:

```json
{
  "api_key": "r4epRxaMzDdmDX",
  "linx_url": "https://put.icu",
  "delim": " | "
}
```

Run `upsauce` with the path to your image as a single command line argument:

```bash
upsauce path/to/image.jpg
```

Alternatively, you can use a direct link to an image (it should end with .jpg, .png, etc.). The image will be downloaded in your current working directory
(for example, [this](https://i.imgur.com/W42kkKS.jpg) I got from [RustNAO](https://github.com/ClementTsang/RustNAO) example):

```bash
upsauce https://i.imgur.com/W42kkKS.jpg
```

You will receive output like this:

```bash
Skipped ext_url: "https://chan.sankakucomplex.com/post/show/5874087" # This source is not included in the next Markdown string

# Markdown string. All links SauceNAO found + linx-server file link
[Pixiv](https://www.pixiv.net/member_illust.php?mode=medium&illust_id=61477678) | [Twitter](https://twitter.com/i/web/status/837653407900934145) | ... | [Image](https://put.icu/x2zj493c.jpeg)
https://put.icu/s/x2zj493c.jpeg # Direct link to linx-server file

To delete your file on `https://put.icu` use: `curl -H "Linx-Delete-Key: Ypzwq5tT81UkLUiwYuEYXQ5oPWOHaw" -X DELETE https://put.icu/x2zj493c.jpeg` # Delete uploaded file if you no longer need it.
```

> [!NOTE]
> If SauceNAO finds several links to the same site, only the first link found will be printed in the Markdown string, the rest are marked as skipped.
