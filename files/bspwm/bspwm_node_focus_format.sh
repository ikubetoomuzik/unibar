#! /bin/sh

DEFAULT_DIR=$(dirname $(realpath $0));
cd $DEFAULT_DIR;

bspc subscribe node_focus |
  while read -r line;
  do
    echo $line | awk '{ print $4 }' > $DEFAULT_DIR/focus
  done 
