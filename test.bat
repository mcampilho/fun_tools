@echo off
rem Builds every tool and runs a short demo of each one.
rem Needs the MSVC build tools in the environment (e.g. devcmd.bat).
setlocal
cargo build --release || exit /b 1
set "PATH=%~dp0target\release;%PATH%"

rfetch -o
rcowsay "Rust is fantastic!"
rcowsay -f crab -W 30 "Tools that run on Windows and Linux, written in Rust" | rlolcat -f
rponysay -k unicorn "Rust in pixels!"
rcowthink -f owl "Hmm..."

rfortune
rfortune -s | rcowsay -f cat | rboxes -s round -t rfortune | rlolcat -f

echo Tools in Rust! | rboxes double -a c -w 30
echo Tools in Rust! | rboxes -s bold -c rainbow -t Hello
rboxes --list

rbonsai -s 7
rbonsai -l -M 7
rbonsai -u -f random -l

rmatrix -s 7
rsl -n 3
rsl -u -n 3

rfiglet "fun_tools"
rem (cmd reads .bat files in the OEM code page: no accented letters here)
rfiglet -f pixel -c "Hello, World!"
rtoilet --gay Rust
rtoilet -f tiny -F border:metal "Windows + Linux"
rrig -c 2 -e
rrig -C pt -o csv -c 3
rrig -c 3 | rnms -a

rtty-clock -s -x
rasciiquarium
rasciiquarium -u
rnyancat
rpipes -p 4 -R
rfire -c -f 300
rfire -m braille -p gas -f 300
rhollywood
rgenact --exit-after-time 20
rsudoku -p 1 -d easy
rblocks
rinvaders
rsudoku
endlocal
