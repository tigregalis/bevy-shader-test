# Bevy custom material boilerplate

Minimal custom material fragment shader boilerplate for Bevy, with hot reloading.

`cargo run` or `cargo watch -x run`

The Bevy asset server watches `assets/`, including the shaders e.g. `custom_material.wgsl`, hot reloading the shaders.

cargo-watch watches `src/` and `Cargo.toml` for code changes and restarts the app.

Bevy saves/restores window state (size and position) via `window.json`.
