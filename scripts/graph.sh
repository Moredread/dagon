#!/bin/sh

cargo graph | dot -Tpng -o dagon-deps.png
