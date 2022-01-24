# rustpad
<p align="center">
<a href="https://github.com/Kibouo/rustpad/actions?query=workflow%3A%22Rust+CI%22">
    <img alt="build status shield" src="https://img.shields.io/github/workflow/status/Kibouo/rustpad/Rust%20CI/main?logo=github">
</a>
<a href="https://www.rust-lang.org/">
    <img alt="uses Rust shield" src="https://img.shields.io/badge/uses-Rust-orange?logo=rust">
</a>
<a href="https://github.com/Kibouo/rustpad/blob/main/LICENSE">
    <img alt="license shield" src="https://img.shields.io/github/license/Kibouo/rustpad?color=teal">
</a>
</p>

<p align="center">
<img alt="asciinema example run" src="./assets/example_run.gif">
</p>

## 👇🏃 Download
| <p align="center">Arch linux</p>                                                                                                                                             | <p align="center">Kali / Debian</p>                                                                                                                         | <p align="center">Others</p>                                                                                                                                   |
| ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `yay -Syu rustpad`                                                                                                                                                           | `apt install ./rustpad.deb`                                                                                                                                 | `cargo install rustpad`                                                                                                                                        |
| <p align="center"><a href="https://aur.archlinux.org/packages/rustpad-bin/"><img alt="aur shield" src="https://img.shields.io:/aur/version/rustpad-bin?color=blue"/></a></p> | <p align="center"><a href="https://github.com/Kibouo/rustpad/releases"><img alt="deb shield" src="https://img.shields.io/badge/deb-v1.8.0-purple"/></a></p> | <p align="center"><a href="https://crates.io/crates/rustpad"><img alt="crates.io shield" src="https://img.shields.io:/crates/v/rustpad?color=yellow"/></a></p> |

## 🔪🏛️ A multi-threaded what now?
`rustpad` is a multi-threaded successor to the classic [`padbuster`](https://github.com/AonCyberLabs/PadBuster), written in Rust. It abuses a [Padding Oracle vulnerability](https://en.wikipedia.org/wiki/Padding_oracle_attack) to decrypt any cypher text or encrypt arbitrary plain text **without knowing the encryption key**!

## 🦀💻 Features
- Decryption of cypher texts
- Encryption of arbitrary plain text
- Multi-threading on both block and byte level
- Modern, real-time and interactive TUI!
- No-TTY support, so you can just pipe output to a file
- Supports *Web* server oracles...
- ... and *Script*-based oracles. For when you need just that extra bit of control.
- Automated calibration of web oracle's (in)correct padding response
- Progress bar and automated retries
- Tab auto-completion
- Block-level caching
- Smart detection of cypher text encoding, supporting: `hex`, `base64`, `base64url`
- No IV support
- Written in purely safe Rust, making sure you don't encounter nasty crashes

## 🗒️🤔 Usage
Using `rustpad` to attack a padding oracle is easy. It requires only 4 pieces of information to start:
- type of oracle (`web`/`script`, see below)
- target oracle (`--oracle`)
- cypher text to decrypt (`--decrypt`)
- block size (`--block-size`)

### Web mode
Web mode specifies that the oracle is located on the web. In other words, the oracle is a web server with a URL.

For a padding oracle attack to succeed, an oracle must say so if a cypher text with incorrect padding was provided. `rustpad` will analyse the oracle's responses and automatically calibrate itself to the oracle's behaviour.

```log
; rustpad web --help
rustpad-web 1.8.0
Question a web-based oracle

USAGE:
    rustpad {web, --web, -W} [OPTIONS] --oracle <ORACLE_LOCATION> --block-size <BLOCK_SIZE> --decrypt <CYPHER_TEXT>

OPTIONS:
    -A, --user-agent <USER_AGENT>
            User-agent to identify with

            [default: rustpad/1.8.0]

    -B, --block-size <BLOCK_SIZE>
            Block size used by the cypher

            [options: 8, 16]

    -c, --consider-body
            Consider the response body and content length when determining the web oracle's response to (in)correct padding

    -d, --data <POST_DATA>
            Data to send in a POST request

    -D, --decrypt <CYPHER_TEXT>
            Original cypher text, received from the target service, which is to be decrypted

        --delay <THREAD_DELAY>
            Delay between requests within a thread, in milliseconds

            [default: 0]

    -e, --encoding <ENCODING>
            Specify encoding used by the oracle to encode the cypher text

            [options: auto, hex, base64, base64url]

            [default: auto]

    -E, --encrypt <PLAIN_TEXT>
            Plain text to encrypt. Note: encryption mode requires a cypher text to gather necessary data

    -h, --help
            Print help information

    -H, --header <HEADER>
            HTTP header to send

            [format: <name>:<value>]

    -k, --insecure
            Disable TLS certificate validation

    -K, --keyword <KEYWORD>
            Keyword indicating the location of the cypher text in the HTTP request. It is replaced by the cypher text's value at runtime

            [default: CTEXT]

    -n, --no-iv
            Cypher text does not include an Initialisation Vector

        --no-cache
            Disable reading and writing to the cache file

        --no-url-encode
            Disable URL encoding and decoding of cypher text

    -o, --output <LOG_FILE>
            File path to which log output will be written

    -O, --oracle <ORACLE_LOCATION>
            The oracle to question with forged cypher texts. This can be a URL or a shell script.

            See the subcommands `web --help` and `script --help` respectively for further help.

        --proxy-credentials <PROXY_CREDENTIALS>
            Credentials to authenticate against the proxy server with

            [format: <user>:<pass>]

    -r, --redirect
            Follow HTTP Redirects

    -t, --threads <THREAD_COUNT>
            Amount of threads in the thread pool

            [default: 64]

    -T, --timeout <REQUEST_TIMEOUT>
            Web request timeout in seconds

            [default: 10]

    -v, --verbose
            Increase verbosity of logging

    -V, --version
            Print version information

    -x, --proxy <PROXY_URL>
            Proxy server to send web requests over. Supports HTTP(S) and SOCKS5

Indicate the cypher text's location! See `--keyword` for clarification.
```

### Script mode
Script mode was made for power users ~~or CTF players 🏴‍☠️ who were given a script to run~~. The target oracle is a local shell script.

Scripts allow you to run attacks against local oracles or more exotic services. Or you can use script mode to customise and extend `rustpad`'s features. However, if you're missing a feature, feel free to open an issue on [GitHub](https://github.com/Kibouo/rustpad/issues)!

```log
; rustpad script --help
rustpad-script 1.8.0
Question a script-based oracle

USAGE:
    rustpad {script, --script, -S} [OPTIONS] --oracle <ORACLE_LOCATION> --block-size <BLOCK_SIZE> --decrypt <CYPHER_TEXT>

OPTIONS:
    -B, --block-size <BLOCK_SIZE>
            Block size used by the cypher

            [options: 8, 16]

    -D, --decrypt <CYPHER_TEXT>
            Original cypher text, received from the target service, which is to be decrypted

        --delay <THREAD_DELAY>
            Delay between requests within a thread, in milliseconds

            [default: 0]

    -e, --encoding <ENCODING>
            Specify encoding used by the oracle to encode the cypher text

            [options: auto, hex, base64, base64url]

            [default: auto]

    -E, --encrypt <PLAIN_TEXT>
            Plain text to encrypt. Note: encryption mode requires a cypher text to gather necessary data

    -h, --help
            Print help information

    -n, --no-iv
            Cypher text does not include an Initialisation Vector

        --no-cache
            Disable reading and writing to the cache file

        --no-url-encode
            Disable URL encoding and decoding of cypher text

    -o, --output <LOG_FILE>
            File path to which log output will be written

    -O, --oracle <ORACLE_LOCATION>
            The oracle to question with forged cypher texts. This can be a URL or a shell script.

            See the subcommands `web --help` and `script --help` respectively for further help.

    -t, --threads <THREAD_COUNT>
            Amount of threads in the thread pool

            [default: 64]

    -v, --verbose
            Increase verbosity of logging

    -V, --version
            Print version information

Script must respond with exit code 0 for correct padding, and any other code otherwise. Cypher text is
passed as the 1st argument.
```

### Shell auto-completion
`rustpad` can generate tab auto-completion scripts for most popular shells:
```sh
rustpad setup <shell>
```

Consult your shell's documentation on what to do with the generated script.

## 🕥💤 Coming soon
- [ ] smarter URL parsing
- [ ] advanced calibration: response text should contain "x", time-based
- [ ] automated block size detection
- [ ] improve linux binary's file size
- [ ] .NET URL token encoding?
