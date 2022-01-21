use std::path::PathBuf;

use clap::{AppSettings, Args, Parser, Subcommand};
use reqwest::Url;

use crate::{block::block_size::BlockSize, oracle::oracle_location::OracleLocation};

use super::{
    encoding_option::EncodingOption, header::Header, proxy_credentials::ProxyCredentials,
    request_timeout::RequestTimeout, thread_count::ThreadCount, thread_delay::ThreadDelay,
    user_agent::UserAgent,
};

use getset::Getters;

#[derive(Parser, Getters)]
#[clap(
    about,
    long_about = None,
    version,
    setting =
    AppSettings::SubcommandRequired |
    AppSettings::PropagateVersion |
    AppSettings::DisableHelpSubcommand |
    AppSettings::InferSubcommands
)]
// TODO: some of the following options should be global, such that they can be placed after subcommand specifiers. But required globals are not supported by `clap`
// https://github.com/clap-rs/clap/issues/1546
pub(super) struct Cli {
    #[clap(
        help = "Oracle to question",
        long_help = "The oracle to question with forged cypher texts. This can be a URL or a shell script.

See the subcommands `web --help` and `script --help` respectively for further help.",
        short = 'O',
        long = "oracle",
        aliases = &["oracle", "oracle-location", "oracle_location"],
        // global = true
    )]
    #[getset(get = "pub(super)")]
    oracle_location: OracleLocation,
    #[clap(
        // TODO: let clap list the options
        // https://github.com/clap-rs/clap/issues/3312
        help = "Block size used by the cypher",
        long_help = "Block size used by the cypher

[options: 8, 16]",
        short = 'B',
        long = "block-size",
        aliases = &["block-size", "block_size"],
        // global = true
    )]
    #[getset(get = "pub(super)")]
    block_size: BlockSize,
    #[clap(
        help = "Cypher text to decrypt",
        long_help = "Original cypher text, received from the target service, which is to be decrypted",
        short = 'D',
        long = "decrypt",
        aliases = &["decrypt", "cypher-text", "cypher_text", "ctext"],
        // global = true
    )]
    #[getset(get = "pub(super)")]
    cypher_text: String,
    #[clap(
        help = "Plain text to encrypt",
        long_help = "Plain text to encrypt. Note: encryption mode requires a cypher text to gather necessary data",
        short = 'E',
        long = "encrypt",
        aliases = &["encrypt", "plain-text", "plain_text", "ptext"],
        requires = "cypher-text",
        // because this is global and its dependency is not, if in a subcommand, it will complain about the dependency not being set. This global can be enabled if `cypher_text`'s global is enabled
        // global = true
    )]
    #[getset(get = "pub(super)")]
    plain_text: Option<String>,
    #[clap(
        help = "Cypher text without IV",
        long_help = "Cypher text does not include an Initialisation Vector",
        short = 'n',
        long = "no-iv",
        aliases = &["no-iv", "no_iv", "noiv"],
        global = true
    )]
    #[getset(get = "pub(super)")]
    no_iv: bool,
    #[clap(
        help = "Increase verbosity",
        long_help = "Increase verbosity of logging",
        short = 'v',
        long = "verbose",
        aliases = &["verbose", "verbosity"],
        global = true,
        parse(from_occurrences)
    )]
    #[getset(get = "pub(super)")]
    verbosity: u8,
    #[clap(
        help = "Thread count",
        long_help = "Amount of threads in the thread pool",
        short = 't',
        long = "threads",
        aliases = &["threads", "thread-count", "thread_count"],
        default_value_t = ThreadCount::default(),
        global = true
    )]
    #[getset(get = "pub(super)")]
    thread_count: ThreadCount,
    #[clap(
        help = "Delay between requests within a thread",
        long_help = "Delay between requests within a thread, in milliseconds",
        long = "delay",
        aliases = &["delay", "thread_delay", "thread-delay"],
        default_value_t = ThreadDelay::default(),
        global = true
    )]
    #[getset(get = "pub(super)")]
    thread_delay: ThreadDelay,
    #[clap(
        help = "Output to file",
        long_help = "File path to which log output will be written",
        short = 'o',
        long = "output",
        aliases = &["output", "output_file", "output-file", "log", "log_file", "log-file"],
        global = true
    )]
    #[getset(get = "pub(super)")]
    log_file: Option<PathBuf>,
    #[clap(
        help = "Specify cypher text encoding",
        // TODO: let clap list the options
        // https://github.com/clap-rs/clap/issues/3312
        long_help = "Specify encoding used by the oracle to encode the cypher text

[options: auto, hex, base64, base64url]",
        short = 'e',
        long = "encoding",
        aliases = &[
            "encoding",
            "enc",
            "cypher_text_encoding",
            "cypher-text-encoding",
            "ctext_encoding",
            "ctext-encoding",
            "ctext_enc",
            "ctext-enc"
        ],
        default_value_t = EncodingOption::Auto,
        global = true
    )]
    #[getset(get = "pub(super)")]
    encoding: EncodingOption,
    #[clap(
        help = "Disable URL encoding and decoding of cypher text",
        long = "no-url-encode",
        aliases = &["no-url-encode", "no_url_encode", "no-url-enc", "no_url_enc"],
        global = true
    )]
    #[getset(get = "pub(super)")]
    no_url_encode: bool,
    #[clap(
        help = "Disable cache",
        long_help = "Disable reading and writing to the cache file",
        long = "no-cache",
        aliases = &["no-cache", "no_cache"],
        global = true
    )]
    #[getset(get = "pub(super)")]
    no_cache: bool,

    #[clap(subcommand)]
    pub(super) sub_command: SubCommand,
}

#[derive(Subcommand, Debug)]
pub(super) enum SubCommand {
    #[clap(
        about = "Question a web-based oracle",
        long_about = None,
        after_help = "Indicate the cypher text's location! See `--keyword` for clarification.",
        display_order = 1,
        short_flag = 'W',
        long_flag = "web"
    )]
    Web(WebCli),
    #[clap(
        about = "Question a script-based oracle",
        long_about = None,
        after_help = "Script must respond with exit code 0 for correct padding, and any other code otherwise. Cypher text is passed as the 1st argument.",
        display_order = 2,
        short_flag = 'S',
        long_flag = "script"
    )]
    Script(ScriptCli),
}

#[derive(Args, Getters, Debug)]
pub(super) struct WebCli {
    #[clap(from_global)]
    pub(super) thread_delay: ThreadDelay,
    #[clap(
        help = "Data to send in a POST request",
        short = 'd',
        long = "data",
        aliases = &["data", "post-data"]
    )]
    pub(super) post_data: Option<String>,
    #[clap(
        help = "HTTP header to send",
        long_help = "HTTP header to send

[format: <name>:<value>]",
        short = 'H',
        long = "header",
        multiple_occurrences = true,
        number_of_values = 1
    )]
    pub(super) header: Vec<Header>,
    #[clap(help = "Follow HTTP Redirects", short = 'r', long = "redirect")]
    pub(super) redirect: bool,
    #[clap(
        help = "Disable TLS certificate validation",
        short = 'k',
        long = "insecure",
        aliases = &["no_cert_check", "insecure-tls", "no-cert-check", "no-tls-check"]
    )]
    pub(super) no_cert_validation: bool,
    #[clap(
        help = "Keyword indicating the cypher text",
        long_help = "Keyword indicating the location of the cypher text in the HTTP request. It is replaced by the cypher text's value at runtime",
        short = 'K',
        long = "keyword",
        default_value = "CTEXT"
    )]
    pub(super) keyword: String,
    #[clap(
        help = "Consider the body during calibration",
        long_help = "Consider the response body and content length when determining the web oracle's response to (in)correct padding",
        short = 'c',
        long = "consider-body",
        aliases = &["consider_body", "consider-body", "consider-content", "consider_content"]
    )]
    pub(super) consider_body: bool,
    #[clap(
        help = "User-agent to identify with",
        short = 'A',
        long = "user-agent",
        aliases = &["user-agent", "user_agent"],
        default_value_t = UserAgent::default()
    )]
    pub(super) user_agent: UserAgent,
    #[clap(
        help = "Proxy server",
        long_help = "Proxy server to send web requests over. Supports HTTP(S) and SOCKS5",
        short = 'x',
        long = "proxy",
        aliases = &["proxy", "proxy_server", "proxy-server", "proxy_url", "proxy-url"]
    )]
    pub(super) proxy_url: Option<Url>,
    #[clap(
        help = "Credentials for proxy server",
        long_help = "Credentials to authenticate against the proxy server with

[format: <user>:<pass>]",
        long = "proxy-credentials",
        aliases = &["proxy-credentials", "proxy_credentials", "proxy_creds", "proxy-creds"],
        requires = "proxy-url"
    )]
    pub(super) proxy_credentials: Option<ProxyCredentials>,
    #[clap(
        help = "Web request timeout",
        long_help = "Web request timeout in seconds",
        short = 'T',
        long = "timeout",
        aliases = &["timeout", "request_timeout", "request-timeout", "timeout_secs", "timeout_seconds"],
        default_value_t = RequestTimeout::default()
    )]
    pub(super) request_timeout: RequestTimeout,
}

#[derive(Args, Debug)]
pub(super) struct ScriptCli {
    #[clap(from_global)]
    pub(super) thread_delay: ThreadDelay,
}
