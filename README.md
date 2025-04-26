# rdp-channel-client

## Purpose

This utility is inspired by my day job struggles with RDP virtual channel development. 
It's relatively tricky to find known-good code examples for working with these.
The intention is to provide an executable example that works out of the box with the ECHO virtual channel 
(which is available by default on the Windows RDP server implementation) and can be dynamically configured
to test other virtual channels.

It will support bi-directional communications with these channels, however as of now it has only very rudimentary 
RDP functionality that I knocked up earlier today!

## Building

It is implemented in Rust; all dependencies should be fetched and built by cargo, meaning that it should not be
necessary to worry about providing dependencies such as GUI toolkits. A mere `cargo build --release` should be sufficient to get up and running.

## Usage 

```
Usage: rdp-channel-client.exe [OPTIONS] --username <USERNAME> --password <PASSWORD> <HOST>

Arguments:
  <HOST>

Options:
  -u, --username <USERNAME>
  -p, --password <PASSWORD>
  -d, --domain <DOMAIN>
  -P, --port <PORT>          [default: 3389]
  -h, --help                 Print help
```

The command line arguments should be self explanatory. The intention is to deprecate the `password` argument in favour of either reading
passwords from an environment variable or prompting via the GUI. 
