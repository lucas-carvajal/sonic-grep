# Sonic Grep

A version of grep, just written in Rust and much faster.
At least that's the goal. Polishing this while I learn more and more about Rust.
The initial setup is based on Chapter 12 of The Rust Programming Language.

### Additions made on top of the original example
1. Process lines in parallel workers, to speed up processing

### Ideas for what's next
1. Investigate how grep works and how to optimize it
2. Directory Searches - search all files in a directory in parallel
3. Implement additional features (e.g. output line numbers)
