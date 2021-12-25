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
| <p align="center"><a href="https://aur.archlinux.org/packages/rustpad-bin/"><img alt="aur shield" src="https://img.shields.io:/aur/version/rustpad-bin?color=blue"/></a></p> | <p align="center"><a href="https://github.com/Kibouo/rustpad/releases"><img alt="deb shield" src="https://img.shields.io/badge/deb-v1.5.0-purple"/></a></p> | <p align="center"><a href="https://crates.io/crates/rustpad"><img alt="crates.io shield" src="https://img.shields.io:/crates/v/rustpad?color=yellow"/></a></p> |

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
- Smart detection of cypher text encoding, supporting: `hex`, `base64`, `base64url`
- No IV support
- Written in purely safe Rust, making sure you don't encounter nasty crashes

## 🗒️🤔 Usage
Using `rustpad` to attack a padding oracle is easy. It requires only 4 pieces of information to start:
- target oracle (`--oracle`)
- cypher text to decrypt (`--decrypt`)
- block size (`--block-size`)
- type of oracle (`web`/`script`, see below)

```log
; rustpad --help
rustpad
Multi-threaded Padding Oracle attacks against any service.

USAGE:
    rustpad [OPTIONS] --block-size <block_size> --decrypt <decrypt> --oracle <oracle> <SUBCOMMAND>

OPTIONS:
    -B, --block-size <block_size>
            Block size used by the cypher [possible values: 8, 16]

    -D, --decrypt <decrypt>
            Original cypher text, received from the target service, which is to be decrypted

    -E, --encrypt <encrypt>
            Plain text to encrypt. Encryption mode requires a cypher text to gather necessary data

    -h, --help
            Prints help information

    -n, --no-iv
            Cypher text does not include an Initialisation Vector

    -O, --oracle <oracle>
            The oracle to question with forged cypher texts. This can be a URL or a shell script.
            See the subcommands `web --help` and `script --help` respectively for further help.
    -V, --version
            Prints version information

    -v, --verbose
            Increase verbosity of logging


SUBCOMMANDS:
    web       Question a web-based oracle
    script    Question a script-based oracle
```

### Web mode
Web mode specifies that the oracle is located on the web. In other words, the oracle is a web server with a URL.

For a padding oracle attack to succeed, an oracle must say so if a cypher text with incorrect padding was provided. `rustpad` will analyse the oracle's responses and automatically calibrate itself to the oracle's behaviour.

```log
; rustpad web --help
rustpad-web
Question a web-based oracle

USAGE:
    rustpad --block-size <block_size> --decrypt <decrypt> --oracle <oracle> web [OPTIONS]

OPTIONS:
    -c, --consider-body
            Consider the response body and content length when determining the web oracle's response to (in)correct padding

    -d, --data <data>
            Data to send in a POST request

    -h, --help
            Prints help information

    -H, --header <header>...
            HTTP header to send

    -k, --insecure
            Disable TLS certificate validation

    -K, --keyword <keyword>
            Keyword indicating the location of the cypher text in the HTTP request. It is replaced by the cypher text's value at runtime [default: CTEXT]

    -n, --no-iv
            Cypher text does not include an Initialisation Vector

    -r, --redirect
            Follow 302 Redirects

    -A, --user-agent <user_agent>
            User-agent to identify with [default: rustpad/<version>]

    -v, --verbose
            Increase verbosity of logging


Indicate the cypher text's location! See `--keyword` for clarification.
```

### Script mode
Script mode was made for power users ~~or CTF players 🏴‍☠️ who were given a script to run~~. The target oracle is a local shell script.

Scripts allow you to run attacks against local oracles or more exotic services. Or you can use script mode to customise and extend `rustpad`'s features. However, if you're missing a feature, feel free to open an issue on [GitHub](https://github.com/Kibouo/rustpad/issues)!

```log
; rustpad script --help
rustpad-script
Question a script-based oracle

USAGE:
    rustpad --block-size <block_size> --decrypt <decrypt> --oracle <oracle> script [OPTIONS]

OPTIONS:
    -h, --help
            Prints help information

    -n, --no-iv
            Cypher text does not include an Initialisation Vector

    -v, --verbose
            Increase verbosity of logging


Script must respond with exit code 0 for correct padding, and any other code otherwise. Cypher text is passed as the 1st argument.
```

## 🕥💤 Coming soon
- [ ] override/specify encoding
- [ ] caching mechanism
- [ ] tab auto-complete
- [ ] smarter URL parsing
- [ ] advanced calibration: response text should contain "x"
- [ ] automated block size detection
- [ ] improve linux binary's file size
- [ ] .NET URL token encoding?
