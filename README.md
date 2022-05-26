# multiple_socket_exchanges

This is simple project which connect binance, coinbase and okex socket.

The cache mode should connect via socket for 10 seconds only, disconnect and print “cache complete” to the terminal.

Save result of the aggregate and the data points used to create the aggregate to a file.

The read mode should simply read and print the file to the screen.

Project execution:
- Install packages and build project using `cargo build` from project root directory.
- Cache pairs data using this command `./target/debug/simple --mode=cache --pairs=btc_usdt` or `cargo run -- --mode=cache --pairs=btc_usdt`. (here we can define multiple pairs using "," ex. `--pairs=btc_usdt,eth_usdt`)
- Read and aggregate pairs data and show to user using this command `./target/debug/simple --mode=read` or `cargo run -- --mode=read`.