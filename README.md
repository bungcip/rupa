# Rupa

**A compiled, LLM-friendly graphics programming language for static vector and raster artwork.**

Rupa turns readable `.rupa` source files into deterministic PNG images. It combines declarative drawing primitives with a small procedural language, strict validation, and stable diagnostics — so both humans and language models get fast, predictable feedback.

```rupa
size 320px x 220px
background "#f4f1ea"

rectangle at (20px, 20px)
    size 90px x 60px
    fill "#7950F2"
    stroke white
    width 3px

circle center at (200px, 50px)
    radius 30px
    fill orange
```

[**Try the playground**](https://bungcip.github.io/rupa/playground/) · [Language specification](rupa-specification.md) · [Examples](examples/)

The playground runs the real compiler and renderer in the browser through WebAssembly. Nothing is uploaded, and the preview matches `rupa render`.

---

## Why Rupa

- **LLM-friendly** — one canonical spelling for each concept, predictable indentation, explicit defaults.
- **Human-readable** — concise English keywords and a regular visual structure.
- **Verifiable** — strict grammar, semantic validation, stable error codes, and suggested fixes.
- **Composable** — reusable `group` templates placed many times with independent transforms.
- **Hybrid** — vector shapes, gradients, patterns, text, layers, and raster pixel work in one document.
- **Deterministic** — the same source, resources, and seed produce byte-identical PNGs.

## Features

| Area | What is supported |
|------|-------------------|
| Shapes | rectangles, circles, ellipses, triangles, polygons, lines, polylines, paths |
| Fills | named colours, hex colours, gradients, tiled patterns |
| Composition | layers, z-index, clipping, opacity, blend modes, reusable groups |
| Text | vendored `Rupa Sans` regular and bold faces, no host-font dependency |
| Raster | layer pixel buffers and raster drawing operations |
| Procedural | variables, expressions, conditionals, loops, functions, seeded randomness |
| Tooling | `check`, `render`, `fmt`, `inspect`, `metadata`, `bench`, JSON diagnostics |

## Install and run

Rupa requires a recent stable Rust toolchain (edition 2024).

```bash
git clone https://github.com/bungcip/rupa.git
cd rupa

# Check, format, and render
cargo run -p rupa-cli -- check examples/shapes.rupa
cargo run -p rupa-cli -- fmt examples/shapes.rupa
cargo run -p rupa-cli -- render examples/shapes.rupa -o shapes.png

# Inspect the compiled structure as JSON
cargo run -p rupa-cli -- inspect examples/layers.rupa
```

Install the CLI on your `PATH`:

```bash
cargo install --path crates/rupa-cli
rupa render examples/text.rupa -o text.png
```

Use `rupa --help` for all commands. Diagnostics include a stable code, a `file:line:column` location, an explanation, and a suggested correction where possible.

## Run the playground locally

The playground needs a WebAssembly build of the compiler and renderer:

```bash
bash playground/build.sh
node playground/serve.mjs
# → http://127.0.0.1:5173/
```

The `wasm32-unknown-unknown` target is installed automatically if missing. Serving over HTTP is required because browsers reject streaming WebAssembly from `file://` URLs.

## Repository layout

```
crates/
  rupa-syntax/     lexer, parser, AST, formatter
  rupa-compiler/   name resolution, validation, lowering
  rupa-ir/         graphics IR and validation
  rupa-geometry/   paths, transforms, geometry
  rupa-renderer/   CPU rendering and compositing
  rupa-image/      image resources and pixel buffers
  rupa-wasm/       browser bindings (C ABI)
  rupa-cli/        `rupa` command-line tool
playground/        static editor + live preview
site/              landing page deployed to GitHub Pages
examples/          valid and invalid `.rupa` fixtures
tests/golden/      golden PNG references
docs/              language reference and decision records
agent/             development roadmap, tasks, and handoffs
```

## Documentation

- [Language specification](rupa-specification.md) — the authoritative language reference.
- [Playground guide](playground/README.md) — editor features, WASM ABI, and limits.
- [Language reference JSON](docs/language-reference.json) — canonical metadata consumed by the CLI and playground.
- [Architecture decisions](docs/decisions/) — recorded design decisions and their consequences.
- [Roadmap](agent/ROADMAP.md) and [project status](agent/STATUS.md) — implementation history.

## Development

```bash
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo build --workspace
cargo test --workspace
```

CI runs the same checks on every push and pull request. The workspace has **500+ tests**, including golden-image comparisons, executable examples, diagnostic fixtures, and WASM output parity.

## Deployment

Pushing to `main` triggers `.github/workflows/pages.yml`, which builds `rupa-wasm`, assembles the landing page and playground, and publishes them to GitHub Pages. Enable **Settings → Pages → Source: GitHub Actions** once for the repository. The expected URLs are:

- Homepage: `https://<owner>.github.io/rupa/`
- Playground: `https://<owner>.github.io/rupa/playground/`

## Version

**0.1.0** — the first public release. The language and file format may still evolve before 1.0; `.rbc` bytecode and additional output formats are not implemented.

## License

Rupa is dual-licensed under either of

- [Apache License, Version 2.0](LICENSE-APACHE)
- [MIT License](LICENSE-MIT)

at your option. Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project shall be dual-licensed as above, without any additional terms or conditions.

The bundled `Rupa Sans` font is licensed under the SIL Open Font License; see [crates/rupa-renderer/assets/OFL.txt](crates/rupa-renderer/assets/OFL.txt) and [the vendored-font decision](docs/decisions/DECISION-040-vendored-font.md).
