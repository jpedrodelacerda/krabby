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
      local cmd=$"(krabby cd ${@:2})"
      eval "echo $cmd"
      eval $cmd
      ;;
    * )
      local cmd=$"(krabby cd ${@:1})"
      eval "echo $cmd"
      eval $cmd
      ;;
  esac
}
