use anyhow::Result;
use getset::Getters;
use reqwest::Proxy;

use super::{
    cli::WebCli, header::Header, request_timeout::RequestTimeout, thread_delay::ThreadDelay,
    user_agent::UserAgent,
};

#[derive(Debug, Clone, Getters)]
pub struct WebConfig {
    // arguments
    #[getset(get = "pub")]
    post_data: Option<String>,
    #[getset(get = "pub")]
    headers: Vec<Header>,
    #[getset(get = "pub")]
    keyword: String,
    #[getset(get = "pub")]
    user_agent: UserAgent,
    #[getset(get = "pub")]
    proxy: Option<Proxy>,
    #[getset(get = "pub")]
    request_timeout: RequestTimeout,
    #[getset(get = "pub")]
    thread_delay: ThreadDelay,

    // flags
    #[getset(get = "pub")]
    redirect: bool,
    #[getset(get = "pub")]
    insecure: bool,
    #[getset(get = "pub")]
    consider_body: bool,
}

impl TryFrom<WebCli> for WebConfig {
    type Error = anyhow::Error;

    fn try_from(cli: WebCli) -> Result<Self> {
        Ok(Self {
            post_data: cli.post_data,
            headers: cli.header,
            keyword: cli.keyword,
            user_agent: cli.user_agent,
            proxy: cli
                .proxy_url
                .map(|url| -> Result<Proxy> {
                    let proxy = Proxy::all(url)?;
                    if let Some(proxy_creds) = cli.proxy_credentials {
                        Ok(proxy.basic_auth(proxy_creds.username(), proxy_creds.password()))
                    } else {
                        Ok(proxy)
                    }
                })
                .transpose()?,
            request_timeout: cli.request_timeout,
            thread_delay: cli.thread_delay,
            redirect: cli.redirect,
            insecure: cli.no_cert_validation,
            consider_body: cli.consider_body,
        })
    }
}
