#!/bin/bash

rebuild=false

while [[ $# > 0 ]]; do
  opt="$(echo "${1/#--/-}" | tr "[:upper:]" "[:lower:]")"
  case "$opt" in
    -rebuild|-r)
      rebuild=true
      ;;
    esac

  shift
done

if [[ "$rebuild" == true ]]; then
    cargo clean
fi

cargo build $@
