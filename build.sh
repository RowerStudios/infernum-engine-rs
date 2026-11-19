cargo build #build game

bin=$(sed -n '/^\[\[bin\]\]/,/^\[/s/^name *= *"\(.*\)"/\1/p' Cargo.toml | head -n 1) # detect binary name

cp "target/debug/$bin" "build/$bin" #copy game file from debug/build to build