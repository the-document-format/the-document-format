demo_file := "/tmp/tdf-demo.tdf"

write-demo format="json":
    cargo run --bin tdf-cli -- write {{demo_file}} --format {{format}}

read-demo format="json":
    cargo run --bin tdf-cli -- read {{demo_file}} --format {{format}}
