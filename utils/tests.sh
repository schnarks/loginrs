#/bin/bash
SCRIPTPATH="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
cd $SCRIPTPATH
cargo run --bin user
cargo run --bin session
cargo run --bin settings
cargo run --bin issue_helpers