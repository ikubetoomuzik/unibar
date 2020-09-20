#! /bin/zsh
# Trying to replace awk script.

bspc subscribe report node_focus |
  while read line
  do
    setopt shwordsplit
    if [ "${line:0:10}" = "node_focus" ]; then
      printf "${line:33}" > "bspwm/focus"
    else
      line="${line:1}"
      line="${line//:/ }"
      for tmp in $line
      do
        unsetopt shwordsplit
        ch="${tmp:0:1}"
        dktp="${tmp:1}"
        case "$ch" in
          M|m)
            if [ -z "$count" ]; then
              count=1;
            fi
            array[$count]="new_desktop"
            count=$(($count + 1));
            array[$count]=$dktp;
            count=$(($count + 1));
            ;;
          O|F|U)
            array[$count]=" {B0}{H0}{F1}  $dktp   {/BHF}";
            count=$(($count + 1));
            ;;
          o|u)
            array[$count]=" {F1}  $dktp   {/F}";
            count=$(($count + 1));
            ;;
          f)
            array[$count]="  $dktp  ";
            count=$(($count + 1));
            ;;
        esac
      done
      next_is_mon=0
      for i in $array
      do
        if [ "$i" = "new_desktop" ]; then
          if ! [ -z "$filename" ] && ! [ -z "$printstr" ]; then
            printf "$printstr" > "bspwm/$filename"
            printstr=""
          fi
          next_is_mon=1
        elif [ $next_is_mon -eq 1 ]; then
          filename="$i"
          next_is_mon=0
        elif [ -z "$printstr" ]; then
          printstr="$i"
        else
          printstr+="$i"
        fi
      done
      printf "$printstr" > "bspwm/$filename"
    fi
  done
  
