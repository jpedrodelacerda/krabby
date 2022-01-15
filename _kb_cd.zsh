kb () {
    case $1 in
      run | add )
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
