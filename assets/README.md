# Asset Plan (P1+)

P1 currently runs with text labels, but static assets are already imported from local `lishogi`.

## Imported now

- Pieces:
  - `assets/pieces/standard/western` (30 SVG)
  - `assets/pieces/standard/ryoko_1kanji` (30 SVG)
- Boards:
  - `assets/boards/lishogi` (11 files, jpg/png)
- Sounds:
  - `assets/sounds/lishogi/shogi` (15 OGG files)

## Source paths

- Pieces:
  - `/home/harry/Play/lishogi/ui/@build/pieces/assets/standard/western`
  - `/home/harry/Play/lishogi/ui/@build/pieces/assets/standard/ryoko_1kanji`
- Boards:
  - `/home/harry/Play/lishogi/ui/@build/static/assets/images/boards`
- Sounds:
  - `/home/harry/Play/lishogi/ui/@build/static/assets/sound/ogg/system/shogi`

## Re-import command

```bash
./script/import_assets_from_lishogi.sh /home/harry/Play/lishogi .
```

## Notes

- Keep original filenames for easy mapping with existing lishogi naming conventions.
- If you ship externally, add a license note file for third-party assets.
