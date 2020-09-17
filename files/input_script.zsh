#! /bin/zsh

DEFAULT_DIR=$(dirname $(realpath $0));
SPACING="   ";

# start the bspwm watching script here.
if ! pgrep -x bspwm_report_fo > /dev/null; then
  $DEFAULT_DIR/bspwm/bspwm_report_format.sh &
fi

# If the exit file already exists then clear it.
FILE="$HOME/.config/unibar/stop"
if [ -f "$FILE" ]; then
  rm "$FILE";
fi

# Sleep to start because the bar needs time to start up.
sleep 1;

# Set the count to 0 to start.
count=0;

while [ 1 -gt 0 ]; do
  # Skip defaults to false.
  skip=1

  cd $DEFAULT_DIR/bspwm;
  # Get desktops
  new_dktps=$(< "$1");
  if [ "$dktps" != "$new_dktps" ]; then
    # If they are different than we set the new value.
    dktps=$new_dktps;
    # Also need to inform the later loop to reprint string.
    skip=0;
  fi
  # Go back;
  cd -;

  # Get new time.
  # only every second.
  # Grab the current time.
  new_date=$(date +%H:%M);
  if [ "$date" != "$new_date" ]; then
    # If it is different than we set the new value.
    date=$new_date;
    # Also need to inform the later loop to reprint string.
    skip=0;
  fi

  # Check the root file system usage.
  # Every 60 seconds.
  if [ $((count % 60)) -eq 0 ]; then
    # Grab the current percentage of root partition.
    new_root_percent=$(df --output=pcent,target | grep "/$" | awk '{print $1}');
    if [ "$root_percent" != "$new_root_percent" ]; then
      # If it is different than we set the new value.
      root_percent=$new_root_percent;
      # Also need to inform the later loop to reprint string.
      skip=0;
    fi
  fi

  # If skip is set to false then we re-print the string.
  if [ $skip -eq 0 ]; then
    # Clear the string.
    unset string;

    # Set the left aligned section.
    string="{}$dktps<|>";
    # Set the center aligned section.
    string+="<|>";
    # Set the right aligned section.
    string+="/{F1}:{/F} $root_percent$SPACING{H0} 83%  {/H}$SPACING{H0}{F1} $date{/H}{/F}";

    # Print the string.
    echo "$string";
  fi

  # Check for the exit file and if it's there we exit.
  if [ -f "$FILE" ]; then
    echo "QUIT NOW";
    exit;
  fi

  # Increment our conter variable.
  count=$((count + 1));

  # Sleep so we don't eat the processor.
  sleep 0.1;
done
