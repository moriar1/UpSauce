# UpSauce

UpSauce uploads your image on [linx-server](https://github.com/ZizzyDizzyMC/linx-server/) instance and
outputs links to it found on [SauceNAO](https://saucenao.com) as Markdown string, like this: [Pixiv](https://www.pixiv.net/member_illust.php?mode=medium&illust_id=61477678) | [Twitter](https://twitter.com/i/web/status/837653407900934145) | ... | [Image](https://put.icu/x2zj493c.jpeg)

# Usage
Sing up on [SauceNAO](https://saucenao.com) and put your API key in `config.json`:

```json
{
  "api_key": "r4epRxaMzDdmDX"
}
```

Copy your image to upload to the git project directory
(for instance, [this](https://i.imgur.com/W42kkKS.jpg) I got from [RustNao](https://github.com/ClementTsang/RustNAO) example):

```bash
cp path/to/your/image.jpg . 
```

Then run:
```bash
cargo run -- image.jpg
```

You will get output like this:
```bash
Skipped ext_url: "https://chan.sankakucomplex.com/post/show/5874087" # This source is not included in the next Markdown string

# Markdown string. All links SauceNAO found + linx-server file link
[Pixiv](https://www.pixiv.net/member_illust.php?mode=medium&illust_id=61477678) | [Twitter](https://twitter.com/i/web/status/837653407900934145) | ... | [Image](https://put.icu/x2zj493c.jpeg)
https://put.icu/s/x2zj493c.jpeg # Direct link to linx-server file

To delete your file on `https://put.icu` use: `curl -H "Linx-Delete-Key: Ypzwq5tT81UkLUiwYuEYXQ5oPWOHaw" -X DELETE https://put.icu/x2zj493c.jpeg` # Delete uploaded file if you no longer need it.
```

> [!NOTE]
> If SauceNAO finds several links to the same site, then only the first link found is prints in Markdown string, the rest are marked as skipped.
