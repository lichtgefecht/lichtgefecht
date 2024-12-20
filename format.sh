#!/bin/bash

find . -name 'deps' -prune -o -name 'build' -prune -o \( -name '*.c' -o -name '*.h' \) -type f -exec clang-format -i {} \;