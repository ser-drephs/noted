#!/bin/bash

coverage=false
lcov=false

while [[ $# > 0 ]]; do
  opt="$(echo "${1/#--/-}" | tr "[:upper:]" "[:lower:]")"
  case "$opt" in
    -coverage|coverage)
      coverage=true
      ;;
    -lcov|lcov)
      lcov=true
      ;;
    esac

  shift
done

if [[ "$coverage" == true ]]; then
    if [[ "$lcov" == true ]]; then
        cargo tarpaulin --out 'Lcov'
    else
        cargo tarpaulin --out 'Html'
    fi
else
    cargo test $@
fi