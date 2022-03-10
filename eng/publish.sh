#!/bin/bash

linux=false
windows=false

while [[ $# > 0 ]]; do
  opt="$(echo "${1/#--/-}" | tr "[:upper:]" "[:lower:]")"
  case "$opt" in
    -linux|linux)
      linux=true
      ;;
    -windows|windows)
      windows=true
      ;;
    esac

  shift
done

args=("")

if [[ "$linux" == true && "$windows" == false ]]; then
    target=x86_64-unknown-linux-gnu
fi

if [[ "$linux" == false && "$windows" == true ]]; then
    target=x86_64-pc-windows-gnu
fi

cargo build --target $target $@

if [[ "$windows" == true ]]; then
    publishdir=./out/windows/debug
    publishdir2=./out/windows/release

    mkdir -p $publishdir
    mkdir -p $publishdir2
    cp debug/target/$target/debug/$app.exe $publishdir
    cp debug/target/$target/release/$app.exe $publishdir2
fi