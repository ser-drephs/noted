#!/bin/bash

set -u

set -e

function usage_fn () {
  echo "Common settings:"
  echo "  --help                 Print help and exit (short: -h)"
  echo ""

  echo "Actions:"
  echo "  --build                Build (short: -b)"
#   echo "  --rebuild                  Rebuild solution"
  echo "  --test                 Run all tests (short: -t)"
  echo "  --release              Build release"
  echo "  --publish              Publish artifact"
  echo ""

  echo "Advanced settings:"
  echo "  --linux                Build for Linux (x86_64-unknown-linux-gnu)"
  echo "  --windows              Build for Windows (x86_64-pc-windows-gnu)"
  echo ""
}

source="${BASH_SOURCE[0]}"
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
build_dir=$SCRIPT_DIR/../debug/target/

build=false
rebuild=false
test=false
coverage=false
lcov=false
release=false
publish=false

linux=false
windows=false

publishdir=
outdir=
target=
output=debug
app=noted

while [[ $# > 0 ]]; do
  opt="$(echo "${1/#--/-}" | tr "[:upper:]" "[:lower:]")"
  case "$opt" in
    -help|-h)
      usage_fn
      exit 0
      ;;
    -build|-b)
      build=true
      ;;
    -rebuild|-r)
      rebuild=true
      ;;
    -test|-t)
      test=true
      ;;
    -coverage)
      coverage=true
      ;;
    -lcov)
      lcov=true
      ;;
    -release)
      release=true
      ;;
    -publish)
      publish=true
      ;;
    -linux)
      linux=true
      ;;
    -windows)
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

if [[ "$build" == true || "$rebuild" == true ]]; then
    args+=(build)
    if [[ "$linux" == true || "$windows" == true ]]; then
        args+=(--target $target)
    fi

    if [[ "$release" == true ]]; then
        output=release
        args+=(--release)
    fi

    if [[ "$rebuild" == true ]]; then
        rm -rf $build_dir
    fi
elif [[ "$test" == true ]]; then
    if [[ "$coverage" == true ]]; then
        if [[ "$lcov" == true ]]; then
            args+=(tarpaulin --out 'Lcov')
        else
            args+=(tarpaulin --out 'Html')
        fi
    else
        args+=(test)
    fi
fi

if [[ ${args[@]} > 0 ]]; then
    echo "Execute cargo${args[@]}"
    cargo ${args[@]}
fi

if [[ "$publish" == true ]]; then
    version=$(cat ../Cargo.toml | grep "^version = " | cut -d '=' -f2 | xargs)
    version=$(echo "$version" | tr -d '\r')
    echo Version=$version

    if [[ "$linux" == true && "$windows" == false ]]; then
        target=x86_64-unknown-linux-gnu
        publishdir=$SCRIPT_DIR/../out/linux/$version/$output
        outdir=$build_dir/$target/$output/$app
    fi

    if [[ "$linux" == false && "$windows" == true ]]; then
        target=x86_64-pc-windows-gnu
        publishdir=$SCRIPT_DIR/../out/windows/$version/$output
        outdir=$build_dir/$target/$output/$app.exe

    fi

    mkdir -p $publishdir
    cp $outdir $publishdir
fi
