function format(str) {
  tmp = substr(str, 0, 1);
  name = substr(str, 2);
  switch (tmp) {
    case /(F|O|U)/:
      return sprintf(" {B0}{H0}{F1}  %s   {/BHF}", name)
    case "o":
      return sprintf(" {F1}  %s   {/F}", name)
    default:
      return sprintf("  %s  ", name)
  }
}

function secondbit(word, idx) {
    first = substr(word, 0, 1);
    switch (first) {
      case /(M|m)/:
        return idx;
      default:
        return 0;
    }
}

BEGIN { FS=":" };

{
  firstmon = substr($1,3);
  if (firstmon != "eDP-1") {
    printf("%s%s%s%s%s", format($2), format($3), format($4), format($5), format($6)) > sprintf("%s", substr($1, 3)); 
    for (i=7; i<=NF; i++) {
      tempvar = (secondbit($(i), i));
      if (tempvar != 0) {
        second_var = i;
        break;
      }
    }
    printf("%s%s%s%s%s", format($(second_var + 1)), format($(second_var + 2)), format($(second_var + 3)), format($(second_var + 4)), format($(second_var + 5))) > sprintf("%s", substr($second_var, 2)); 
  } else {
    printf("%s%s%s%s%s%s%s%s%s%s", format($2), format($3), format($4), format($5), format($6), format($7), format($8), format($9), format($10), format($11))  > sprintf("%s", substr($1, 3)); 
  }
  exit; 
}
