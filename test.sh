#!/bin/sh
# Builds every tool and runs a short demo of each one.
set -e
cd "$(dirname "$0")"
cargo build --release
PATH="$PWD/target/release:$PATH"

rfetch -o
rcowsay "Rust is fantastic!"
rcowsay -f crab -W 30 "Tools that run on Windows and Linux, written in Rust" | rlolcat -f
rcowthink -f owl "Hmm..."

rfortune
rfortune -s | rcowsay -f cat | rboxes -s round -t rfortune | rlolcat -f

echo "Tools in Rust!" | rboxes double -a c -w 30
echo "Tools in Rust!" | rboxes -s bold -c rainbow -t Hello
rboxes --list

rbonsai -s 7
rbonsai -l -M 7

rmatrix -s 7
rsl -n 3
