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
BEGIN { FS=":" };
{  printf("%s%s%s%s%s", format($2), format($3), format($4), format($5), format($6)) > sprintf("%s", substr($1, 3)); 
printf("%s%s%s%s%s", format($11), format($12), format($13), format($14), format($16)) > sprintf("%s", substr($10, 2)); exit; }
