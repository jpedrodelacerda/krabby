#!/bin/env bash

kb() {
  if [[ $# -lt 1 ]];then
     krabby help
     return
  fi

  local RESERVED_TOKENS=(
    'run'
    'shell'
    'project'
    'script'
    'help'
    'hook'
    '-h'
    '--help'
    '-d'
    '--database'
    '-f'
    '--project-file'
    '-V'
    '--version'
  )

  __kb_cd() {
    cmd=$(krabby cd $@)
    # We check if the directory exists before we 'cd' into it.
    if [[ "$?" -eq 0 ]]; then
      eval $cmd
    else
      echo $cmd
      return 1
    fi
  }

  case $1 in
    # We have to filter out both 'run' and 'cd' commands so we can evaluate.
    'run' | 'r' )
      cmd="$(krabby run ${@:2})"
      eval $cmd
      ;;
    'cd' )
      __kb_cd "${@:2}"
      ;;
    * )
      if [[ "${RESERVED_TOKENS[*]}" =~ "$1" ]]; then
        krabby $@
      else
        __kb_cd "${@:1}"
      fi
      ;;
  esac
}
