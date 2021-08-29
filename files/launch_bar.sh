#! /bin/sh

cargo run -- "$@" < <($HOME/.config/unibar/input_$1.zsh) 
