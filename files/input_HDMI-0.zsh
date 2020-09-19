#! /bin/zsh

DEFAULT_DIR=$(dirname $(realpath $0));
SPACING="    ";

# start the bspwm watching script here.
if ! pgrep -x bspwm_report_fo > /dev/null; then
  $DEFAULT_DIR/bspwm/bspwm_report_format.sh &
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

  ##########################################################
  # Get desktops
  new_dktps=$(< "$DEFAULT_DIR/bspwm/HDMI-0");
  if [ "$dktps" != "$new_dktps" ]; then
    # If they are different than we set the new value.
    dktps=$new_dktps;
    # Also need to inform the later loop to reprint string.
    skip=0;
  fi
  ##########################################################

  # only every second.
  ##########################################################
  if [ $(($count % 10)) -eq 0 ]; then
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
  fi
  ##########################################################

  # If skip is set to false then we re-print the string.
  # OR every three seconds to make sure that if the screen sleeps the bar comes back.
  if [ $skip -eq 0 ] || [ $count -lt 0 ] || [ $(($count % 30)) -eq 0 ]; then
    # Clear the string and set to just empty brackets 
    # so we don't lose leading spaces.
    string="{}"

    # Set the left aligned section.
    string+="$dktps";
    # Add the section seperator.
    string+="<|>"
    # Add the section seperator.
    string+="<|>"
    # Add the right side.
    string+="$SPACING$time_display";

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
