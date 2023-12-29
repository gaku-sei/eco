# Eco, an e-book toolbox

The repository host multiple cli and gui that allows you to edit, convert, merge, and more, e-book files (cbz, epub, pdf, etc...).

## Tools (with supported format):

- `eco-converter` - cli - Convert e-books to any format (from pdf, mobi, and DRM-free azw3, to cbz only for now)
- `eco-merge` - cli - Merge e-books together when it makes sense (cbz)
- `eco-pack` - cli - pack images into an e-book file (cbz)
- `eco-viewer` - gui - A dead simple e-book reader (cbz)

## Eco Converter

Converts e-books from \* to \* (only pdf, mobi, and DRM-free azw3 to cbz supported for the moment):

```bash
eco-converter "archive.azw3" --from azw3 --outdir out
```

## Eco Merge (cbz only for now)

This will look for all the e-books in `path` and which file name contains `something` and merge them into `output/merged_archive.cbz`:

```bash
eco-merge --archives-glob "path/**/*something*" --outdir "output" --name "merged_archive"
```

## Eco Pack (cbz only for now)

Takes all the `png` files under `source` and pack them into the `archive.cbz` file:

```bash
eco-pack "source/*.png" --name archive --autosplit
```

Options include:

- `--autosplit`: split in 2 landscape images
- `--contrast`: change contrast
- `--brightness`: change brightness

## Eco Viewer (cbz only for now)

View any e-book file with this simple gui:

```bash
eco-viewer "my_archive.cbz"
```
