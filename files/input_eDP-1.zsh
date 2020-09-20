#! /bin/zsh

DEFAULT_DIR=$(dirname $(realpath $0));
SPACING="     ";

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

# start the bspwm watching script here.
if ! pgrep -x bspwm_report_fo > /dev/null; then
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
    if [ ${#focus_win_name} -gt 90 ]; then
      win_name="$focus_win_name[1,87]...";
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
    ip a | grep -e ".*inet .*eno1$" | tr "/" " " | read x new_ip x;
    if [ "$ip" != "$new_ip" ]; then
      ip=$new_ip;
      if [ "$ip" = "" ]; then
        ip_display="{B0}{F1}{/F} disconnected.{/B}"
      else
        ip_display="{F1} $ip{/F}";
      fi
      skip=0;
    fi
    ########################################################
  fi
  ##########################################################

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
    string+="$focus_win_display";
    # Add the section seperator.
    string+="<|>"
    # Set the right aligned section.
    string+="  $root_percent$SPACING$home_percent$SPACING$vol_display$SPACING$ip_display$SPACING$time_display";

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
