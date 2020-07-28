#!/bin/bash

if [ -n "$1" -a -f "$1" ]
   then chown root:sudo "$1" ; chmod g+r "$1"
fi
