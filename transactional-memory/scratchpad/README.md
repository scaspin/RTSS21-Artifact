# Scratchpad

This scratchpad directory contains examples of transactions in different control
flow. We compile these files individually and inspect their MIR by hand to check
that the compiler inserts lock and unlock calls correctly.

1. Make some change: either
    - Create or modify new `*.rs` test case
    - Modify and build Rust compiler
1. Run `make`
1. Read corresponding MIR in `*.mir` and output log in `*.log`

