# ProText-RS

Minimal Rust implementation of ProText using crossterm.

Features (minimal):
- Open a file: `protext-rs <file>`
- Save: Ctrl+S
- Quit: Ctrl+C or 'q' (with modified confirmation behavior)
- Basic navigation and editing (arrow keys, backspace, enter)
- Read-only detection

Build

Requires Rust and Cargo.

cargo build --release

The resulting binary will be at `target/release/protext_rs`.

Notes
- This is a small, direct translation of the Python ProText's core features. It doesn't yet implement find/replace prompts; adding them would be straightforward.
- Nuitka is a Python-to-binary tool and is not applicable to Rust. Cargo produces native binaries directly.
