This directory contains tiny fonts used by documentation examples.

Fonts were made using `pyftsubset` tool from the [fonttools](https://github.com/fonttools/fonttools) package:

```
pyftsubset SourceSansPro-Regular.ttf --output-file=SourceSansPro-Regular-Tiny.ttf \
    --gids=0-80,1100-1110,1780-1824 --notdef-glyph --notdef-outline --recommended-glyphs
```

### Origin

- SourceSansPro-Regular.ttf - SIL OFL 1.1

https://github.com/adobe-fonts/source-sans-pro

