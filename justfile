alias d := run_debug
alias p := profile
alias pd := profile_debug

run $RUST_LOG="debug":
    cd sample_project && cargo run --release

run_debug $RUST_LOG="debug":
    cd sample_project && cargo run

profile $CARGO_PROFILE_RELEASE_DEBUG="true":
    cd sample_project && cargo flamegraph
    firefox sample_project/flamegraph.svg

profile_debug $CARGO_PROFILE_RELEASE_DEBUG="true":
    cd sample_project && cargo flamegraph --profile dev
    firefox sample_project/flamegraph.svg
