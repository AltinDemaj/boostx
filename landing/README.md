# BoostX landing page

Static marketing site for BoostX downloads.

## Deploy on Vercel

1. Create a new Vercel project and set **Root Directory** to `landing`.
2. Framework preset: **Other** (no build command).
3. Deploy — `index.html` is served at `/` and the installer at `/download/BoostX-Setup.exe`.

`vercel.json` sets `Content-Disposition: attachment` for `.exe` files under `/download/`.

## Refresh the installer

From the repository root (Linux with NSIS + `cargo-xwin`):

```bash
npm run build:installer
```

This cross-compiles the Windows NSIS bundle and copies it to `landing/download/BoostX-Setup.exe`.
