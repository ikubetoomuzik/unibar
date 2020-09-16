#! /bin/sh

DEFAULT_DIR=$(dirname $(realpath $0));

cd $DEFAULT_DIR;

input_string.sh $1 | cargo run -- $1;


