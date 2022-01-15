kb () {
  if [[ $# -lt 1 ]];then
     krabby help
     return
  fi
  case $1 in
    # We filter every command other than `cd` which is the only 'corner case'
    run | add | help )
      krabby $@
      ;;
    'cd' )
      eval $"(krabby cd \"${@:2}\")"
      ;;
    *)
      eval $"(krabby cd \"${@:1}\")"
      ;;
  esac
}
