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
rustc --edition 2021 -C panic=abort baremin_ls.rs -o baremin_ls
rustc --edition 2021 -C panic=abort baremin_cat.rs -o baremin_cat
rustc --edition 2021 -C panic=abort event_bus.rs -o event_bus

# raylib related
rustc --edition 2021 -C panic=abort raylib_demo.rs -o raylib_demo 
rustc --edition 2021 -C panic=abort raylib_calc.rs -o raylib_calc
rustc --edition 2021 -C panic=abort game_breakout.rs -o game_breakout
rustc --edition 2021 -C panic=abort game_snake.rs -o game_snake
rustc --edition 2021 -C panic=abort rl_cube_demo.rs -o rl_cube_demo
rustc --edition 2021 -C panic=abort rl_analog_clock.rs -o rl_analog_clock
rustc --edition 2021 -C panic=abort linalg_demo.rs -o linalg_demo
rustc --edition 2021 -C panic=abort several_3d_shapes.rs -o several_3d_shapes
rustc --edition 2021 -C panic=abort torus_and_cone.rs -o torus_and_cone
rustc --edition 2021 -C panic=abort simple_raylib_counter.rs -o simple_raylib_counter
rustc --edition 2021 -C panic=abort reactive_counter.rs -o reactive_counter
rustc --edition 2021 -C panic=abort event_bus_counter.rs -o event_bus_counter

rustc --edition 2021 -C panic=abort earth_wireframe.rs -o earth_wireframe
rustc --edition 2021 -C panic=abort gp2d_collision_demo.rs -o gp2d_collision_demo






