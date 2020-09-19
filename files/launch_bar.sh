#! /bin/sh

DEFAULT_DIR=$(dirname $(realpath $0));


$DEFAULT_DIR/input_$1.zsh | cargo run -q -- "$@" 


