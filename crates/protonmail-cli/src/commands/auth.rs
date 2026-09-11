//! Authentication commands: login, logout, whoami.

use crate::cli::{ClientPreset, Ctx};
use crate::commands::{prompt_line, resume};
use crate::render;
use proton_core::{Client, Error, LoginOptions, Result, TotpPrompt};
use serde_json::json;
use std::sync::Arc;

fn env_nonempty(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|v| !v.is_empty())
}

pub async fn login(ctx: &Ctx) -> Result<()> {
    let username = match env_nonempty("PROTON_USER") {
        Some(u) => u,
        None => prompt_line("Proton username/email: ")?,
    };
    let password = match env_nonempty("PROTON_PASSWORD") {
        Some(p) => p,
        None => rpassword::prompt_password("Proton password: ")?,
    };

    // Explicit flags win; otherwise fall back to a --client preset.
    let app_version = ctx
        .app_version
        .clone()
        .or_else(|| ctx.client.map(|c| c.app_version().to_string()));
    let user_agent = resolve_user_agent(ctx.user_agent.clone(), ctx.client);

    let opts = LoginOptions {
        username,
        password,
        totp: ctx.totp.clone(),
        mailbox_password: ctx.mailbox_password.clone(),
        profile: ctx.profile.clone(),
        base_url: ctx.api_url.clone(),
        app_version,
        user_agent: Some(user_agent),
        hv: Some(crate::hv::resolver(
            ctx.captcha_token.clone(),
            ctx.api_url
                .clone()
                .unwrap_or_else(|| "https://mail.proton.me/api".to_string()),
            ctx.captcha_chrome,
        )),
    };

    let client = Client::login_with_totp_prompt(opts, totp_prompt()).await?;
    let email = client.primary_email().unwrap_or("(unknown)");
    if ctx.json {
        render::json_out(&json!({ "status": "ok", "email": email }));
    } else {
        println!("Logged in as {email}");
    }
    Ok(())
}

/// Explicit `--user-agent` wins, then a `--client` preset, then an honest
/// `protonmail-cli/<version> (<os>)`.
fn resolve_user_agent(explicit: Option<String>, preset: Option<ClientPreset>) -> String {
    explicit
        .or_else(|| preset.map(|c| c.user_agent().to_string()))
        .unwrap_or_else(|| {
            format!(
                "protonmail-cli/{} ({})",
                env!("CARGO_PKG_VERSION"),
                std::env::consts::OS
            )
        })
}

/// Ask for the 2FA code only once the server requires it, so it is still
/// valid after a CAPTCHA.
fn totp_prompt() -> TotpPrompt {
    Arc::new(|| {
        Box::pin(async {
            let code =
                tokio::task::spawn_blocking(|| rpassword::prompt_password("Proton 2FA code: "))
                    .await
                    .map_err(|e| Error::Other(format!("2FA prompt: {e}")))??;
            Ok(code.trim().to_string())
        })
    })
}

pub async fn logout(ctx: &Ctx) -> Result<()> {
    let client = resume(&ctx.profile).await?;
    client.logout().await?;
    if ctx.json {
        render::json_out(&json!({ "status": "ok" }));
    } else {
        println!("Logged out");
    }
    Ok(())
}

pub async fn whoami(ctx: &Ctx) -> Result<()> {
    let client = resume(&ctx.profile).await?;
    let email = client.primary_email().unwrap_or("(unknown)");
    if ctx.json {
        render::json_out(&json!({ "email": email }));
    } else {
        println!("{email}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_agent_defaults_to_honest_cli_identity() {
        assert_eq!(
            resolve_user_agent(None, None),
            format!(
                "protonmail-cli/{} ({})",
                env!("CARGO_PKG_VERSION"),
                std::env::consts::OS
            )
        );
    }

    #[test]
    fn user_agent_explicit_then_preset_win() {
        assert_eq!(
            resolve_user_agent(Some("custom/1".into()), Some(ClientPreset::Web)),
            "custom/1"
        );
        assert_eq!(
            resolve_user_agent(None, Some(ClientPreset::Ios)),
            ClientPreset::Ios.user_agent()
        );
    }
}
