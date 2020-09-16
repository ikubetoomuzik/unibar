#! /bin/sh

cd $HOME/notes/programming/rust/unibar/default/bspwm;

bspc subscribe report |
  while read -r line;
  do
    tmp=$(echo $line | awk -f bspwm.awk);
  done 
