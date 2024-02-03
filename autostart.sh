#!/bin/bash
killall picom
picom &

killall polybar
polybar &

setxkbmap us,ru; setxkbmap -option 'grp:win_space_toggle'

~/.fehbg
