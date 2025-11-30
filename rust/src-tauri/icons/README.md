# App Icons

This directory contains all the app icons generated from `rusty-g6.svg`.

## Source
- **rusty-g6.svg** - Original vector source (keep this to regenerate icons)

## Generated Icons (with transparency)

### Core Icons
- **32x32.png** - Small size for taskbars
- **128x128.png** - Medium size
- **128x128@2x.png** - Retina/HiDPI medium (256x256)
- **icon.png** - Large size (512x512)
- **icon.ico** - Windows executable icon (multi-size)
- **icon.icns** - macOS app icon bundle

### Windows Store Logos
- Square30x30Logo.png through Square310x310Logo.png
- StoreLogo.png

## Regenerating Icons

If you update `rusty-g6.svg`, regenerate all icons with:

```bash
cd rust/src-tauri/icons

# Core PNG icons
convert -background none rusty-g6.svg -resize 32x32 32x32.png
convert -background none rusty-g6.svg -resize 128x128 128x128.png
convert -background none rusty-g6.svg -resize 256x256 128x128@2x.png
convert -background none rusty-g6.svg -resize 512x512 icon.png

# Windows ICO
convert -background none rusty-g6.svg -define icon:auto-resize=256,128,96,64,48,32,16 icon.ico

# macOS ICNS
convert -background none rusty-g6.svg -resize 1024x1024 temp.png
convert temp.png icon.icns
rm temp.png

# Windows Store logos
convert -background none rusty-g6.svg -resize 30x30 Square30x30Logo.png
convert -background none rusty-g6.svg -resize 44x44 Square44x44Logo.png
convert -background none rusty-g6.svg -resize 71x71 Square71x71Logo.png
convert -background none rusty-g6.svg -resize 89x89 Square89x89Logo.png
convert -background none rusty-g6.svg -resize 107x107 Square107x107Logo.png
convert -background none rusty-g6.svg -resize 142x142 Square142x142Logo.png
convert -background none rusty-g6.svg -resize 150x150 Square150x150Logo.png
convert -background none rusty-g6.svg -resize 284x284 Square284x284Logo.png
convert -background none rusty-g6.svg -resize 310x310 Square310x310Logo.png
convert -background none rusty-g6.svg -resize 50x50 StoreLogo.png
```

**Important:** Always use `-background none` to preserve transparency!

## Usage

Icons are configured in `tauri.conf.json` and will be automatically used when building the app.
