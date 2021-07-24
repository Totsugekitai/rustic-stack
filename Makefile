### Makefile_template ###

# don't use TAB
.RECIPEPREFIX = >
# change shell to bash
SHELL := bash
# shell flags
.SHELLFLAGS := -eu -o pipefail -c
# one shell for one target rule
.ONESHELL:
# warning undefined variables
MAKEFLAGS += --warn-undefined-variables
# delete intermediate files on error
.DELETE_ON_ERROR:
# delete implicit rules
MAKEFLAGS += -r

# MAKEFILE_DIR is directory Makefile located in
MAKEFILE_DIR := $(dir $(abspath $(lastword $(MAKEFILE_LIST))))

### Makefile_template end ###

.PHONY: gen-tap test

gen-tap:
> sudo ip tuntap add mode tap user $(USER) name tap0
> sudo ip addr add 192.0.2.1/24 dev tap0
> sudo ip link set tap0 up

test:
> cargo test -- --test-threads=1
