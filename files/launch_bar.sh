#! /bin/sh

pathname=$(dirname $0);

cargo run -- -c $pathname/$1.conf "$@" < <($pathname/input_$1.zsh) 
