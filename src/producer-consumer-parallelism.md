# Producer Consumer Parallelism Feature
1. Main thread launches N threads (default 1, configurable via CLI options)
2. Main thread reades in lines from the file and sending them via a shared channel to the worker threads for processing. When the reader is done reading in all lines, it drops the its sender, thus letting the workers know that the input is done. Then, after processing all open messages, all workers will also drop their senders, thus letting the main thread know that the processing is done.
3. Worker threads send results via channel to main thread.
4. Main thread sorts all received results by line number, and prints them out in order.

### Creating Channels
Create a work channel and a result channel, to be used for communication between the threads.
Use the `crossbeam` create for better functionality, and create bounded channels for the work to manage backpressure.
For the result channel, an unbounded channel should suffice.
Clone the senders and receivers so only one is shared accross all threads.

### Creating worker threads
For the number of workers, clone the receiver and sender. Then put the whole config (incl. search term) into an `Arc<Config>` so it can be shared by all threads, and then spawn the threads, moving the cloned items ownership to the threads.

### Reading in lines
Read in one line at a time and reuse a single string to read it in, to avoid allocations.
Trim the read in line (e.g. remove newlines) and then send it together with the line number to the work channel (clone the string).

### Worker thread processing logic
Use a for loop on the worker thread receiver to read in messages and when a line matches, send it to the results channel.

### Collect results
In the main thread, iterate over the results channel receiver and collect all messages. Then sort all results by line number and print them out.
