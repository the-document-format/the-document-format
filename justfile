demo_file := "/tmp/tdf-demo.tdf"

demo-write format="json":
    cargo run --bin tdf-cli -- write {{demo_file}} --format {{format}}

demo-read format="json":
    cargo run --bin tdf-cli -- read {{demo_file}} --format {{format}}

demo format="json": (demo-write format) (demo-read format)

# kept for backwards compat
demo-demo format="json": (demo-read format)

demo-builder backend="json":
    cargo run --bin tdf-interactive -- --backend {{backend}}

demo-builder-all base="output":
    cargo run --bin tdf-interactive -- --all {{base}}
