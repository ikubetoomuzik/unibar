#! /bin/sh

DEFAULT_DIR=$(dirname $(realpath $0));
cd $DEFAULT_DIR;

bspc subscribe report |
  while read -r line;
  do
    tmp=$(echo $line | awk -f bspwm.awk);
  done 
