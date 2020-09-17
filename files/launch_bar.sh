#! /bin/sh

DEFAULT_DIR=$(dirname $(realpath $0));


$DEFAULT_DIR/input_script.zsh "$1" | cargo run -- "$@" 


