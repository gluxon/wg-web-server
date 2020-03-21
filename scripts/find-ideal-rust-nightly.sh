#!/bin/bash

# Not all Rust nightly builds have the desired packages for this project. This is a simple script
# that finds the last nightly build all packages built successfully on.
#
# Caveats:
#   - The date command has to be GNU coreutils.
#   - Some builds may produce false negatives if the TOML value orderings change.

packages=(
  'clippy-preview.target.x86_64-unknown-linux-gnu'
  'pkg.rls-preview.target.x86_64-unknown-linux-gnu'
)

checkBuild() {
  def=`curl https://static.rust-lang.org/dist/$1/channel-rust-nightly.toml`
  echo

  for package in $packages
  do
    echo "$def" | grep -A 1 -F "[pkg.$package]" | grep 'available = true' &> /dev/null
    status=$?
    if [ $status -ne 0 ]; then
      return $status;
    fi
  done

  return 0
}

bold=$(tput bold)
normal=$(tput sgr0)

for i in {0..99}; do
  build=`date -d "-${i} day" '+%Y-%m-%d'`
  echo "Checking ${bold}$build${normal}"
  if checkBuild $build; then
    echo "Use ${bold}$build${normal}"
    break
  fi
done

