# UpSauce

UpSauce uploads your image to a [linx-server](https://github.com/ZizzyDizzyMC/linx-server/)
instance and retrieves, links found on [SauceNAO](https://saucenao.com) formatting them as a Markdown string:
[Pixiv](https://www.pixiv.net/member_illust.php?mode=medium&illust_id=61477678) | [Twitter](https://twitter.com/i/web/status/837653407900934145) | [Danbooru](https://danbooru.donmai.us/post/show/2631423) | [Gelbooru](https://gelbooru.com/index.php?page=post&s=view&id=3561216) | [Image](https://put.icu/x2zj493c.jpeg)

# Usage

## JSON

Sign up on [SauceNAO](https://saucenao.com) to obtain your API key. Then enter this key,
your linx-server instance URL, and optionally your source delimiter
(default is ` | `) in `config.json`:

```json
{
  "api_key": "r4epRxaMzDdmDX",
  "linx_url": "https://put.icu",
  "delim": " | "
}
```

Run `upsauce` with the path to your image as a single command-line argument:

```bash
upsauce path/to/image.jpg
```

Alternatively, you can provide a direct link to an image (ending with `.jpg`, `.png`, etc.).
The image will be downloaded to your current directory.
For example, this picture from the [RustNAO](https://github.com/ClementTsang/RustNAO) README:

```bash
upsauce https://i.imgur.com/W42kkKS.jpg
```

You will receive output like this:

```bash
# Your markdown string. All links SauceNAO found + link to linx-server file
[Pixiv](https://www.pixiv.net/member_illust.php?mode=medium&illust_id=61477678) | [Twitter](https://twitter.com/i/web/status/837653407900934145) | [Danbooru](https://danbooru.donmai.us/post/show/2631423) | [Gelbooru](https://gelbooru.com/index.php?page=post&s=view&id=3561216) | [Image](https://put.icu/x2zj493c.jpeg)

# Direct link to linx-server file
https://put.icu/s/x2zj493c.jpeg

# This source is not included in the Markdown string
Skipped ext_url: "https://chan.sankakucomplex.com/post/show/5874087"

# `curl` request to delete the uploaded file. Save it somewhere if you intend to revoke access to your image later.
To delete your file on `https://put.icu` use: `curl -H "Linx-Delete-Key: Ypzwq5tT81UkLUiwYuEYXQ5oPWOHaw" -X DELETE https://put.icu/x2zj493c.jpeg`
```

> [!NOTE]
> If SauceNAO finds multiple links to the same site, only the first link
> will be included in the Markdown string, the rest of them are shown in skipped section.

## CLI

Instead of `config.json` file, you can use CLI arguments:

```bash
upsauce --api-key r4epRxaMzDdmDX --linx-url https://put.icu --delim " / " image.jpg
```

### Extra flags

- `-c, --catbox` Upload the file to [catbox.moe](https://catbox.moe) instead of a linx-server instance.
- `-s, --skip-upload` If a URL is provided, upsauce skips uploading the image to the file sharing
service (linx-server or catbox), but still prints the sources.

---

> [!NOTE]
> You can find linx-server instnces in the [List of File Sharing Services](https://gist.github.com/moriar1/5779024379f973f57211d567efe1713b)
