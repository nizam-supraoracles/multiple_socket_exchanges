# multiple_socket_exchanges

This is simple project which connect binance, coinbase and okex socket.

The cache mode should connect via socket for 10 seconds only, disconnect and print “cache complete” to the terminal.

Save result of the aggregate and the data points used to create the aggregate to a file.

The read mode should simply read and print the file to the screen.

Project execution:
- Install packages and build project using this command `cargo build` from project root directory.
- Cache pairs data using this command `./target/debug/application --mode=cache --pairs=btc_usdt` or `cargo run -- --mode=cache --pairs=btc_usdt`. (here we can define multiple pairs using "," ex. `--pairs=btc_usdt,eth_usdt`)
- Read and aggregate pairs data and show to user using this command `./target/debug/application --mode=read` or `cargo run -- --mode=read`.

Project execution in release:
- Build project using this command `cargo build --release`.
- Cache pairs data using this command `./target/release/application --mode==cache --pairs=btc_usdt`
- Read and aggregate pairs data and show to user using this command `./target/release/application --mode=read`

Test Cases:
- Here I have write test cases in "ws_socket/src/test" file.
- Use `cargo test` command to test all cases.
- If you want to test single test case then use like this `cargo test {function_name}`. example `cargo test check_valid_pairs`.

Help:
- If you need any help related to argument, use `--help` to get details of argument
- `./target/release/application --help` or `./target/release/application --help`