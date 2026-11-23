# CLAUDE.md — wire-drop (PUBLIC repository)

This repository will be public. Treat every byte as world-readable forever.

## Hard privacy rules
- Never commit personal data: no real names, addresses, IPs, MAC addresses, GPS, tokens, private emails, host-specific absolute paths, or private infrastructure details.
- Examples must be synthetic and generic.
- The app must work 100% client-side: no analytics, trackers, external fonts/CDNs, telemetry, server component, or runtime network requests.
- User-entered values stay in the browser.

## Build and scope
- Rust stable, wasm32-unknown-unknown, Trunk, plain wasm-bindgen/web-sys only.
- Keep the product small: one core feature and at most two small extras.
- Run `cargo test`, `trunk build --release`, and `bash tools/privacy_check.sh`.
