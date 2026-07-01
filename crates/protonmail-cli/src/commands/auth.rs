//! Authentication commands: login, logout, whoami.

use crate::cli::Ctx;
use crate::commands::{prompt_line, resume};
use crate::render;
use proton_core::{Client, LoginOptions, Result};
use serde_json::json;

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
    let user_agent = ctx
        .user_agent
        .clone()
        .or_else(|| ctx.client.map(|c| c.user_agent().to_string()));

    let opts = LoginOptions {
        username,
        password,
        totp: ctx.totp.clone(),
        mailbox_password: ctx.mailbox_password.clone(),
        profile: ctx.profile.clone(),
        base_url: ctx.api_url.clone(),
        app_version,
        user_agent,
        hv: Some(crate::hv::resolver(
            ctx.captcha_token.clone(),
            ctx.api_url
                .clone()
                .unwrap_or_else(|| "https://mail.proton.me/api".to_string()),
            ctx.captcha_chrome,
        )),
    };

    let client = Client::login(opts).await?;
    let email = client.primary_email().unwrap_or("(unknown)");
    if ctx.json {
        render::json_out(&json!({ "status": "ok", "email": email }));
    } else {
        println!("Logged in as {email}");
    }
    Ok(())
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
