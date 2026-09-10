#!/usr/bin/env zsh


llhelp(){

    >&2 printf '👆 %-40s 👉 %s \n' "\$rxedit_line_number" "$rxedit_line_number"
}

env | grep -i rust 

source missing_script.sh

lhello(){
    echo hello world
}