#! /bin/zsh

BRIGHT_DIR="/sys/class/backlight/intel_backlight";
DEFAULT_DIR="$HOME/.config/unibar";
SPACING="    ";


# Args:
# $1 => drive name to search for.
# $2 => display to use for drive ('/' for root, or '~' for $HOME)
# $3 => old val for storage.
fs_storage() {
  # forces a query of all drives to set up the next queries.
  # Grab the current percentage of root partition.
  new_percent=$(df --output=pcent,target | grep "$1\$" | awk '{printf $1}');
  if [ "$3" != "$new_percent" ]; then
    # If it is different than we set the new value.
    if ! [ -z "$new_percent" ]; then
      echo -n "{F2} $2{F1}: $new_percent{/F}"
    else
      echo -n "{F1}{F2} $2{/F}: disconnected"
    fi
  fi
}

# Args: 
# $1 => Network device name.
net_info() {
  case "${1:0:1}" in
    e)
      ip a | grep -e ".*inet .*$1$" | tr "/" " " | read _ new_info _;
      connected_sym=""
      disconnected_sym=""
      ;;
    w)
      iw dev "$1" info | grep "ssid " | read _ new_info;
      connected_sym=""
      disconnected_sym=""
      ;;
  esac

  if [ -z "$new_info" ]; then
    printf "{F1}$disconnected_sym{/F} disconnected."
  else
    printf "$connected_sym{F1} $new_info{/F}";
  fi
}

# Args none.
brightness() {
  max=$(($(< "$BRIGHT_DIR/max_brightness") - 100));
  cur=$(($(< "$BRIGHT_DIR/brightness") - 100));
  perc=$((100 * $cur / $max));
  if [ $perc -ge 50 ]; then
    spc="";
    sym="";
  elif [ $perc -lt 10 ]; then
    spc=" ";
    sym="";
  else
    spc="";
    sym="";
  fi

  printf "$sym{F1} $spc$perc%%{/F}";
}

# Args none.
battery() {
  acpi | read _ _ chrge bat_per _;
  chrge="${chrge//,}"
  bat_per="${${bat_per//,}//\%}"
  bat_syms=("" "" "");

  case "$chrge" in
    Charging)
      new_count=$(($1 + 1));
      bat_ul=1;
      ;;
    Discharging)
      new_count=$(($1 - 1));
      if [ $bat_per -le 15 ]; then
        bat_ul=3;
      else
        bat_ul=2;
      fi
      ;;
  esac

  bat_sym="${bat_syms[$1]}"

  if [ $bat_per -gt 97 ]; then
    bat_per=100;
    bat_sym="";
  fi

  printf "$new_count{H$bat_ul}$bat_sym{F1} $bat_per%%{/HF}";
}

# start the bspwm watching script here.
if ! pgrep -x bspwm_subscribe > /dev/null; then
  $DEFAULT_DIR/bspwm_subscribe.zsh &
fi

# If the exit file already exists then clear it.
STOP_FILE="$HOME/.config/unibar/stop"
if [ -f "$STOP_FILE" ]; then
  rm "$STOP_FILE";
fi

# Sleep to start because the bar needs time to start up.
sleep 1;

# Set the count to 0 to start.
count=-10;

# before we start the loop, ask unibar to kill us.
echo "PLEASE KILL:$$";

while [ 1 -gt 0 ]; do
  # Skip defaults to false.
  skip=1

  ##########################################################
  # Get current focused window.
  new_focus_win=$(< "$DEFAULT_DIR/bspwm/focus");
  if [ "$focus_win" != "$new_focus_win" ]; then
    focus_win=$new_focus_win;
    skip=0;
  fi
  new_focus_win_name=$(xdotool getwindowname "$focus_win" 2> /dev/null);
  if [ "$focus_win_name" != "$new_focus_win_name" ]; then
    focus_win_name=$new_focus_win_name;
    if [ ${#focus_win_name} -gt 42 ]; then
      win_name="$focus_win_name[1,39]...";
    else
      win_name="$focus_win_name";
    fi
    focus_win_display="{f1}{/f}{F1} $win_name{/F}"
    skip=0;
  fi
  ##########################################################

  ##########################################################
  # Get desktops
  new_dktps=$(< "$DEFAULT_DIR/bspwm/eDP-1");
  if [ "$dktps" != "$new_dktps" ]; then
    # If they are different than we set the new value.
    dktps=$new_dktps;
    # Also need to inform the later loop to reprint string.
    skip=0;
  fi
  ##########################################################

  # Every half second.
  if [ $(($count % 8)) -eq 0 ]; then
    ########################################################
    # Battery bit
    if [ -z "$bat_count" ] || [ $bat_count -gt 3 ]; then
      bat_count=1;
    elif [ $bat_count -lt 1 ]; then
      bat_count=3;
    fi
    return_val=$(battery $bat_count);
    bat_count=${return_val:0:1};
    new_bat_display=${return_val:1};
    if [ "$bat_display" != "$new_bat_display" ]; then
      bat_display=$new_bat_display;
      skip=0;
    fi
    ########################################################
  fi
  # only every second.
  ##########################################################
  if [ $(($count % 10)) -eq 0 ] || [ $count -lt 0 ]; then
    ########################################################
    # Time bit;
    # Grab the current time.
    new_time=$(date +%H:%M);
    # Do the comparison.
    if [ "$cur_time" != "$new_time" ]; then
      # If it is different than we set the new value.
      cur_time=$new_time;
      time_display="{F1} $cur_time{/F}";
      # Also need to inform the later loop to reprint string.
      skip=0;
    fi
    ########################################################
    ########################################################
    # Network bit.
    new_ip_display=$(net_info "wlp58s0");
    if [ "$ip_display" != "$new_ip_display" ]; then
      ip_display=$new_ip_display;
      skip=0;
    fi
    ########################################################
  fi
  ##########################################################

  ######################################
  # Brightness bit.
  new_bright_display=$(brightness);
  if [ "$bright_display" != "$new_bright_display" ]; then
    bright_display=$new_bright_display;
    skip=0;
  fi
  ######################################
  
  ######################################
  # Volume bit.
  # Get current volume.
  new_vol=$(pamixer --get-volume);
  new_mute=$(pamixer --get-mute);
  if [ "$vol" != "$new_vol" ] || [ "$mute" != "$new_mute" ]; then
    # Set the new vol to the vol variable.
    vol=$new_vol;
    # Set the new mute to the mute variable.
    mute=$new_mute;
    if [ "$mute" = "true" ]; then
      vol_display="{F3}muted{/F}";
    else
      if [ $vol -ge 70 ]; then
        vol_icon="";
      elif [ $vol -ge 25 ]; then
        vol_icon="";
      elif [ $vol -eq 0 ]; then
        vol_icon="";
      else
        vol_icon="";
      fi
      if [ $vol -lt 10 ]; then
        vol_space=" ";
      else
        vol_space="";
      fi
      vol_display="{F0}$vol_icon{F1} $vol_space$vol%{/F}";
    fi
    skip=0;
  fi
  ######################################

  ######################################
  # Check the root file system usage.
  # Every 60 seconds.
  if [ $(($count % 60)) -eq 0 ] || [ $count -lt 0 ]; then
    # forces a query of all drives to set up the next queries.
    df -a > /dev/null
    # Grab the current percentage of root partition.
    new_root_percent=$(fs_storage / / $root_percent);
    if [ "$root_percent" != "$new_root_percent" ] && ! [ -z "$new_root_percent" ]; then
      # If it is different than we set the new value.
      root_percent=$new_root_percent;
      # Also need to inform the later loop to reprint string.
      skip=0;
    fi
    ######################################
    new_home_percent=$(fs_storage "/home" "/home" "$home_percent");
    if [ "$home_percent" != "$new_home_percent" ] && ! [ -z "$new_home_percent" ]; then
      # If it is different than we set the new value.
      home_percent=$new_home_percent;
      # Also need to inform the later loop to reprint string.
      skip=0;
    fi
    ######################################
  fi
  ######################################

  # If skip is set to false then we re-print the string.
  if [ $skip -eq 0 ] || [ $count -lt 0 ] || [ $(($count % 30)) -eq 0 ]; then
    # Clear the string and set to just empty brackets 
    # so we don't lose leading spaces.
    string="{}"

    # Set the left aligned section.
    string+="$dktps";
    # Add the section seperator.
    string+="<|>"
    # Set the center aligned section.
    string+="{}";
    # Add the section seperator.
    string+="<|>"
    # Set the right aligned section.
    string+="$focus_win_display$SPACING$root_percent$SPACING$home_percent$SPACING$bright_display$SPACING$vol_display$SPACING$ip_display$SPACING$bat_display$SPACING$time_display";

    # Print the string.
    echo "$string";
  fi

  # Check for the exit file and if it's there we exit.
  if [ -f "$STOP_FILE" ]; then
    # This string makes unibar quit.
    echo "QUIT NOW";
    exit;
  fi

  # Increment our conter variable.
  count=$(($count + 1));
  # Just to make sure we don't get ridiculous.
  if [ $count -gt 1000 ]; then
    count=1
  fi

  # Sleep so we don't eat the processor.
  sleep 0.1;
done
