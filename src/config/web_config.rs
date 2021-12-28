use std::str::FromStr;

use anyhow::{anyhow, Context, Result};
use clap::ArgMatches;
use getset::Getters;
use reqwest::Url;

use super::split_headers;

const VERSION_TEMPLATE: &str = "<version>";
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, Getters)]
pub struct WebConfig {
    // arguments
    #[getset(get = "pub")]
    post_data: Option<String>,
    #[getset(get = "pub")]
    headers: Vec<(String, String)>,
    #[getset(get = "pub")]
    keyword: String,
    #[getset(get = "pub")]
    user_agent: String,
    #[getset(get = "pub")]
    proxy: Option<Url>,
    #[getset(get = "pub")]
    proxy_credentials: Option<(String, String)>,
    #[getset(get = "pub")]
    request_timeout: u64,
    #[getset(get = "pub")]
    thread_delay: u64,

    // flags
    #[getset(get = "pub")]
    redirect: bool,
    #[getset(get = "pub")]
    insecure: bool,
    #[getset(get = "pub")]
    consider_body: bool,
}

impl WebConfig {
    pub(super) fn parse(args: &ArgMatches) -> Result<Self> {
        let keyword = args
            .value_of("keyword")
            .expect("No default value for argument `keyword`");

        Ok(Self {
            post_data: args.value_of("data").map(|data| data.to_owned()),
            headers: match args.values_of("header") {
                Some(headers) => split_headers(headers)?,
                None => vec![],
            },
            keyword: keyword.into(),
            user_agent: args
                .value_of("user_agent")
                .map(|agent| agent.replace(VERSION_TEMPLATE, VERSION))
                .expect("No default value for argument `user_agent`"),
            proxy: args
                .value_of("proxy")
                .map(|proxy| Url::from_str(proxy))
                .transpose()
                .context("Proxy URL failed to parse")?,
            proxy_credentials: args
                .value_of("proxy_credentials")
                .map(|credentials| {
                    let split_credentials = credentials
                        .split_once(':')
                        .map(|(user, pass)| (user.to_owned(), pass.to_owned()));
                    split_credentials.context(format!(
                        "Proxy credentials format invalid! Expected `username:password`, got `{}`.",
                        credentials
                    ))
                })
                .transpose()?,
            request_timeout: args
                .value_of("timeout")
                .map(|timeout| {
                    let timeout = timeout.parse().context("Request timeout failed to parse")?;
                    if timeout > 0 {
                        Ok(timeout)
                    } else {
                        Err(anyhow!("Request timeout must be greater than 0"))
                    }
                })
                .transpose()?
                .expect("No default value for argument `timeout`"),
            thread_delay: args
                .value_of("delay")
                .map(|delay| delay.parse().context("Thread delay failed to parse"))
                .transpose()?
                .expect("No default value for argument `delay`"),

            redirect: args.is_present("redirect"),
            insecure: args.is_present("insecure"),
            consider_body: args.is_present("consider_body"),
        })
    }
}
