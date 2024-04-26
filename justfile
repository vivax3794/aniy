alias t := test

run $RUST_LOG="aniy":
    cd sample_project && cargo run --release

test:
    cd sample_project && cargo nextest run
