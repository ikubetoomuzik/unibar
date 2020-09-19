#! /bin/zsh

DEFAULT_DIR=$(dirname $(realpath $0));
SPACING="    ";

# start the bspwm watching script here.
if ! pgrep -x bspwm_report_fo > /dev/null; then
  $DEFAULT_DIR/bspwm/bspwm_report_format.sh &
fi
if ! pgrep -x bspwm_node_focu > /dev/null; then
  $DEFAULT_DIR/bspwm/bspwm_node_focus_format.sh &
fi

# If the exit file already exists then clear it.
STOP_FILE="$HOME/.config/unibar/stop"
if [ -f "$STOP_FILE" ]; then
  rm "$STOP_FILE";
fi

# Sleep to start because the bar needs time to start up.
sleep 1;

# Set the count to 0 to start.
count=0;

while [ 1 -gt 0 ]; do
  # Skip defaults to false.
  skip=1

  ######################################
  # Get current focused window.
  new_focus_win=$(< "$DEFAULT_DIR/bspwm/focus");
  if [ "$focus_win" != "$new_focus_win" ]; then
    focus_win=$new_focus_win;
    skip=0;
  fi
  new_focus_win_name=$(xdotool getwindowname "$focus_win");
  if [ "$focus_win_name" != "$new_focus_win_name" ]; then
    focus_win_name=$new_focus_win_name;
    focus_win_display="{f1}{/f}{F1} $focus_win_name{/F}"
    skip=0;
  fi

  ######################################

  if ! [ -z "$1" ]; then
    cd $DEFAULT_DIR/bspwm;
    # Get desktops
    new_dktps=$(< "$DEFAULT_DIR/bspwm/$1");
    if [ "$dktps" != "$new_dktps" ]; then
      # If they are different than we set the new value.
      dktps=$new_dktps;
      # Also need to inform the later loop to reprint string.
      skip=0;
    fi
    # Go back;
    cd -;
  fi

  # only every second.
  ##########################################################
  if [ $(($count % 10)) -eq 0 ]; then
    ########################################################
    # Time bit;
    # Grab the current time.
    new_date=$(date +%H:%M);
    # Do the comparison.
    if [ "$date" != "$new_date" ]; then
      # If it is different than we set the new value.
      date=$new_date;
      date_display="{F1} $date{/F}";
      # Also need to inform the later loop to reprint string.
      skip=0;
    fi
    ########################################################
    ########################################################
    # Network bit.
    new_ip=$(ip a | grep -e ".*inet .*eno1$" | tr "/" " " | awk '{ printf "%s",$2 }');
    if [ "$ip" != "$new_ip" ]; then
      ip=$new_ip;
      if [ "$ip" = "" ]; then
        ip_display="{B0}{F1}{/F} disconnected.{/B}"
      else
        ip_display="{F1} $ip{/F} ";
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
      vol_display="{B0}  {F1}{/F}muted   {/B}";
    else
      if [ $vol -gt 66 ]; then
        vol_icon="";
      elif [ $vol -gt 33 ]; then
        vol_icon="";
      else
        vol_icon="";
      fi

      if [ $vol -lt 10 ]; then
        vol_space=" ";
      else
        vol_space="";
      fi

      vol_display="{F1}  Vol{F0}$vol_icon{F1} $vol_space$vol%  {/F}";
    fi
    skip=0;
  fi
  ######################################

  ######################################
  # Check the root file system usage.
  # Every 60 seconds.
  if [ $(($count % 600)) -eq 0 ]; then
    # Grab the current percentage of root partition.
    new_root_percent=$(df --output=pcent,target | grep "/$" | awk '{print $1}');
    if [ "$root_percent" != "$new_root_percent" ]; then
      # If it is different than we set the new value.
      root_percent=$new_root_percent;
      root_percent_display="{F2}/{F1}: $root_percent{/F}"
      # Also need to inform the later loop to reprint string.
      skip=0;
    fi
  fi
  ######################################

  # If skip is set to false then we re-print the string.
  if [ $skip -eq 0 ]; then
    # Clear the string and set to just empty brackets 
    # so we don't lose leading spaces.
    string="{}"

    # Set the left aligned section.
    string+="$dktps";
    # Add the section seperator.
    string+="<|>"
    if [ "$1" != "HDMI-0" ]; then
      # Set the center aligned section.
      string+="$focus_win_display";
      # Add the section seperator.
      string+="<|>"
      # Set the right aligned section.
      string+="$root_percent_display$SPACING$vol_display$SPACING$ip_display$SPACING$date_display";
    fi
    # Add the section seperator.
    string+="<|>"
    # Add the right side.
    string+="$SPACING$date_display";

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
