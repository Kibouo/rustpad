# rustpad
<p align="center">
<a href="https://github.com/Kibouo/rustpad/actions?query=workflow%3A%22Rust+CI%22">
    <!-- shield always shows failing. Wait for breaking change issue to be closed: https://github.com/badges/shields/issues/8671 -->
    <!-- <img alt="build status shield" src="https://img.shields.io/github/actions/workflow/status/Kibouo/rustpad/ci.yaml?logo=github"> -->
    <img alt="build status shield" src="https://img.shields.io/github/actions/workflow/status/simple-icons/simple-icons/verify.yml?branch=master&logo=github&label=build">
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
| `yay -Syu rustpad`                                                                                                                                                           | See releases                                                                                                                                 | `cargo install rustpad`                                                                                                                                        |
| <p align="center"><a href="https://aur.archlinux.org/packages/rustpad-bin/"><img alt="aur shield" src="https://img.shields.io:/aur/version/rustpad-bin?color=blue"/></a></p> | <p align="center"><a href="https://github.com/Kibouo/rustpad/releases"><img alt="deb shield" src="https://img.shields.io/badge/deb-v1.8.1-purple"/></a></p> | <p align="center"><a href="https://crates.io/crates/rustpad"><img alt="crates.io shield" src="https://img.shields.io:/crates/v/rustpad?color=yellow"/></a></p> |

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

### Script mode
Script mode was made for power users ~~or CTF players 🏴‍☠️ who were given a script to run~~. The target oracle is a local shell script.

Scripts allow you to run attacks against local oracles or more exotic services. Or you can use script mode to customise and extend `rustpad`'s features. However, if you're missing a feature, feel free to open an issue on [GitHub](https://github.com/Kibouo/rustpad/issues)!

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
- [ ] .NET URL token encoding?
