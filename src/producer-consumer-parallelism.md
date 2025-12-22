## Producer Consumer Parallelism Feature

1. Main thread launches N threads (default 1, configurable via CLI options)
2. Main thread launches reader thread, reading in lines from the file and sending them via channels to the worker threads for processing using a Round Robin
3. Worker threads send results via channel to main thread
