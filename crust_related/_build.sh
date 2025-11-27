#!/usr/bin/bash 

# basics
rustc --edition 2021 -C panic=abort sample.rs -o sample 
rustc --edition 2021 -C panic=abort sysinfo.rs -o sysinfo
rustc --edition 2021 -C panic=abort math.rs -o math
rustc --edition 2021 -C panic=abort string.rs -o string
rustc --edition 2021 -C panic=abort mem_related.rs -o mem_related
rustc --edition 2021 -C panic=abort args_demo.rs -o args_demo
rustc --edition 2021 -C panic=abort hello_world.rs -o hello_world
rustc --edition 2021 -C panic=abort formatting.rs -o formatting
rustc --edition 2021 -C panic=abort manual_mem.rs -o manual_mem
rustc --edition 2021 -C panic=abort simple_ds.rs -o simple_ds

# raylib related
rustc --edition 2021 -C panic=abort raylib_demo.rs -o raylib_demo 
rustc --edition 2021 -C panic=abort raylib_calc.rs -o raylib_calc
rustc --edition 2021 -C panic=abort game_breakout.rs -o game_breakout
rustc --edition 2021 -C panic=abort game_snake.rs -o game_snake

