pub(super) mod encoding_option;
mod global_config;
pub(super) mod header;
pub(super) mod proxy_credentials;
pub(super) mod request_timeout;
pub(super) mod thread_count;
pub(super) mod thread_delay;
pub(super) mod user_agent;

use std::ops::Deref;

use anyhow::Result;
use getset::Getters;
use reqwest::Proxy;

use self::{
    global_config::GlobalConfig, header::Header, request_timeout::RequestTimeout,
    thread_delay::ThreadDelay, user_agent::UserAgent,
};

use crate::cli::{Cli, ScriptCli, SubCommand, WebCli};

/// Application configuration based on processed CLI args.
#[derive(Debug, Getters)]
pub(super) struct Config {
    global_config: GlobalConfig,
    #[getset(get = "pub(super)")]
    sub_config: SubConfig,
}

#[derive(Debug)]
pub(super) enum SubConfig {
    Web(WebConfig),
    Script(ScriptConfig),
}

#[derive(Debug, Clone, Getters)]
pub(super) struct WebConfig {
    #[getset(get = "pub(super)")]
    post_data: Option<String>,
    #[getset(get = "pub(super)")]
    headers: Vec<Header>,
    #[getset(get = "pub(super)")]
    keyword: String,
    #[getset(get = "pub(super)")]
    user_agent: UserAgent,
    #[getset(get = "pub(super)")]
    proxy: Option<Proxy>,
    #[getset(get = "pub(super)")]
    request_timeout: RequestTimeout,
    #[getset(get = "pub(super)")]
    redirect: bool,
    #[getset(get = "pub(super)")]
    insecure: bool,
    #[getset(get = "pub(super)")]
    consider_body: bool,
    #[getset(get = "pub(super)")]
    thread_delay: ThreadDelay,
}

#[derive(Debug, Clone, Getters)]
pub(super) struct ScriptConfig {
    #[getset(get = "pub(super)")]
    thread_delay: ThreadDelay,
}

impl TryFrom<Cli> for Config {
    type Error = anyhow::Error;

    fn try_from(cli: Cli) -> Result<Self> {
        match cli.sub_command {
            SubCommand::Web(web_cli) => Ok(Self {
                global_config: GlobalConfig::try_from(web_cli.global_options())?,
                sub_config: SubConfig::Web(WebConfig::try_from(*web_cli)?),
            }),
            SubCommand::Script(script_cli) => Ok(Self {
                global_config: GlobalConfig::try_from(script_cli.global_options())?,
                sub_config: SubConfig::Script(ScriptConfig::try_from(*script_cli)?),
            }),
            _ => unreachable!(
                "Attempted to convert sub-command {:?} into a config.",
                cli.sub_command
            ),
        }
    }
}

impl TryFrom<WebCli> for WebConfig {
    type Error = anyhow::Error;

    fn try_from(cli: WebCli) -> Result<Self> {
        Ok(Self {
            post_data: cli.post_data().clone(),
            headers: cli.header().clone(),
            keyword: cli.keyword().clone(),
            user_agent: cli.user_agent().clone(),
            proxy: cli
                .proxy_url()
                .as_ref()
                .map(|url| -> Result<Proxy> {
                    let proxy = Proxy::all(url.clone())?;
                    if let Some(proxy_creds) = cli.proxy_credentials() {
                        Ok(proxy.basic_auth(proxy_creds.username(), proxy_creds.password()))
                    } else {
                        Ok(proxy)
                    }
                })
                .transpose()?,
            request_timeout: cli.request_timeout().clone(),
            redirect: *cli.redirect(),
            insecure: *cli.no_cert_validation(),
            consider_body: *cli.consider_body(),
            thread_delay: cli.thread_delay().clone(),
        })
    }
}

impl TryFrom<ScriptCli> for ScriptConfig {
    type Error = anyhow::Error;

    fn try_from(cli: ScriptCli) -> Result<Self> {
        Ok(Self {
            thread_delay: cli.thread_delay().clone(),
        })
    }
}

impl Deref for Config {
    type Target = GlobalConfig;

    fn deref(&self) -> &Self::Target {
        &self.global_config
    }
}
