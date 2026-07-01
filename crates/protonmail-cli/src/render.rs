//! Output rendering: human-readable text by default, JSON with `--json`.

use crate::cli::ReadFormat;
use proton_core::{
    AddressInfo, Attachment, Contact, ContactEmail, Conversation, Filter, FullMessage, Label,
    LabelCount, MessageMetadata, Recipient,
};
use serde_json::json;

/// Render per-label counts.
pub fn counts(json_mode: bool, counts: &[LabelCount]) {
    if json_mode {
        json_out(&counts);
        return;
    }
    for c in counts {
        println!(
            "  {:<14} total {:>6}  unread {:>6}",
            label_name(&c.label_id),
            c.total,
            c.unread
        );
    }
}

fn label_name(id: &str) -> String {
    match id {
        "0" => "inbox",
        "1" => "all-drafts",
        "2" => "all-sent",
        "3" => "trash",
        "4" => "spam",
        "5" => "all-mail",
        "6" => "archive",
        "7" => "sent",
        "8" => "drafts",
        "10" => "starred",
        other => other,
    }
    .to_string()
}

/// Render a contacts list.
pub fn contacts_list(json_mode: bool, total: u32, contacts: &[Contact]) {
    if json_mode {
        json_out(&json!({ "total": total, "contacts": contacts }));
        return;
    }
    println!("{} contact(s) (total {total})", contacts.len());
    for c in contacts {
        let emails: Vec<&str> = c.emails.iter().map(|e| e.email.as_str()).collect();
        println!("  {}  {}  [{}]", c.id, c.name, emails.join(", "));
    }
}

/// Render contact email entries.
pub fn contact_emails(json_mode: bool, emails: &[ContactEmail]) {
    if json_mode {
        json_out(&emails);
        return;
    }
    println!("{} email(s)", emails.len());
    for e in emails {
        println!("  {}  {}", e.email, e.name);
    }
}

/// Render the account's sending addresses.
pub fn addresses_list(json_mode: bool, addrs: &[AddressInfo]) {
    if json_mode {
        json_out(&addrs);
        return;
    }
    println!("{} address(es)", addrs.len());
    for a in addrs {
        println!("  {}  {}", a.email, a.id);
    }
}

/// Render a filter list.
pub fn filters_list(json_mode: bool, filters: &[Filter]) {
    if json_mode {
        json_out(&filters.iter().map(filter_json).collect::<Vec<_>>());
        return;
    }
    println!("{} filter(s)", filters.len());
    for f in filters {
        let state = if f.status == 1 { "enabled" } else { "disabled" };
        println!("  [{state}] {}  {}", f.id, f.name);
    }
}

/// Render a newly created filter.
pub fn filter_created(json_mode: bool, f: &Filter) {
    if json_mode {
        json_out(&filter_json(f));
    } else {
        println!("Filter created: {} ({})", f.name, f.id);
    }
}

fn filter_json(f: &Filter) -> serde_json::Value {
    json!({ "ID": f.id, "Name": f.name, "Status": f.status, "Version": f.version })
}

/// Serialize a value as pretty JSON to stdout.
pub fn json_out<T: serde::Serialize + ?Sized>(value: &T) {
    match serde_json::to_string_pretty(value) {
        Ok(s) => println!("{s}"),
        Err(_) => println!("null"),
    }
}

fn fmt_addr(r: &Recipient) -> String {
    if r.name.is_empty() || r.name == r.address {
        r.address.clone()
    } else {
        format!("{} <{}>", r.name, r.address)
    }
}

fn fmt_addrs(list: &[Recipient]) -> String {
    if list.is_empty() {
        "-".to_string()
    } else {
        list.iter().map(fmt_addr).collect::<Vec<_>>().join(", ")
    }
}

/// Format a Unix timestamp as `YYYY-MM-DD HH:MM:SS UTC` (no external deps).
pub fn fmt_time(ts: i64) -> String {
    if ts <= 0 {
        return "-".to_string();
    }
    let days = ts.div_euclid(86_400);
    let secs = ts.rem_euclid(86_400);
    let (y, m, d) = civil_from_days(days);
    let (hh, mm, ss) = (secs / 3600, (secs % 3600) / 60, secs % 60);
    format!("{y:04}-{m:02}-{d:02} {hh:02}:{mm:02}:{ss:02} UTC")
}

// Howard Hinnant's days-from-civil inverse (proleptic Gregorian).
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

fn subject_or_empty(s: &str) -> &str {
    if s.is_empty() {
        "(no subject)"
    } else {
        s
    }
}

pub fn messages_list(json: bool, total: u32, msgs: &[MessageMetadata]) {
    if json {
        json_out(&json!({ "total": total, "messages": msgs }));
        return;
    }
    println!("{} message(s) (total {total})", msgs.len());
    for m in msgs {
        let mark = if m.unread != 0 { "*" } else { " " };
        println!();
        println!("{mark} {}", subject_or_empty(&m.subject));
        println!("    from: {}", fmt_addr(&m.sender));
        println!("    date: {}", fmt_time(m.time));
        if m.num_attachments > 0 {
            println!("    attachments: {}", m.num_attachments);
        }
        println!("    id:   {}", m.id);
    }
}

pub fn conversations_list(json: bool, total: u32, convs: &[Conversation]) {
    if json {
        json_out(&json!({ "total": total, "conversations": convs }));
        return;
    }
    println!("{} conversation(s) (total {total})", convs.len());
    for c in convs {
        let mark = if c.num_unread != 0 { "*" } else { " " };
        println!();
        println!("{mark} {}", subject_or_empty(&c.subject));
        println!("    from:     {}", fmt_addrs(&c.senders));
        println!("    messages: {} ({} unread)", c.num_messages, c.num_unread);
        if c.num_attachments > 0 {
            println!("    attachments: {}", c.num_attachments);
        }
        println!("    id:       {}", c.id);
    }
}

fn print_message_header(m: &MessageMetadata, verdict: &str, mime_type: &str) {
    println!("Subject: {}", subject_or_empty(&m.subject));
    println!("From:    {}", fmt_addr(&m.sender));
    println!("To:      {}", fmt_addrs(&m.to_list));
    if !m.cc_list.is_empty() {
        println!("Cc:      {}", fmt_addrs(&m.cc_list));
    }
    println!("Date:    {}", fmt_time(m.time));
    println!("Verdict: {verdict}");
    println!("Type:    {mime_type}");
    println!("Id:      {}", m.id);
}

pub fn full_message(json: bool, msg: &FullMessage, format: ReadFormat, body_only: bool) {
    if json {
        if body_only {
            json_out(&json!({ "body": msg.body }));
        } else {
            json_out(msg);
        }
        return;
    }
    let verdict = serde_json::to_string(&msg.verdict)
        .ok()
        .map(|s| s.trim_matches('"').to_string())
        .unwrap_or_else(|| "unknown".to_string());
    if body_only || matches!(format, ReadFormat::Raw) {
        print!("{}", msg.body);
        if !msg.body.ends_with('\n') {
            println!();
        }
        return;
    }
    print_message_header(&msg.meta, &verdict, &msg.mime_type);
    if !msg.attachments.is_empty() {
        println!("Attachments:");
        for a in &msg.attachments {
            println!("  - {} ({} bytes, id {})", a.name, a.size, a.id);
        }
    }
    println!();
    print!("{}", msg.body);
    if !msg.body.ends_with('\n') {
        println!();
    }
}

pub fn conversation_read(json: bool, conv: &Conversation, msgs: &[FullMessage]) {
    if json {
        json_out(&json!({ "conversation": conv, "messages": msgs }));
        return;
    }
    println!("Conversation: {}", subject_or_empty(&conv.subject));
    println!(
        "Messages: {} ({} unread)",
        conv.num_messages, conv.num_unread
    );
    println!("Id: {}", conv.id);
    for (i, m) in msgs.iter().enumerate() {
        let verdict = serde_json::to_string(&m.verdict)
            .ok()
            .map(|s| s.trim_matches('"').to_string())
            .unwrap_or_else(|| "unknown".to_string());
        println!("\n--- message {}/{} ---", i + 1, msgs.len());
        print_message_header(&m.meta, &verdict, &m.mime_type);
        println!();
        print!("{}", m.body);
        if !m.body.ends_with('\n') {
            println!();
        }
    }
}

pub fn labels_list(json: bool, labels: &[Label], folders: bool) {
    if json {
        json_out(labels);
        return;
    }
    let kind = if folders { "folder" } else { "label" };
    println!("{} {kind}(s)", labels.len());
    for l in labels {
        let parent = l
            .parent_id
            .as_deref()
            .map(|p| format!("  parent={p}"))
            .unwrap_or_default();
        println!("  {} {}  ({}){}", l.id, l.name, l.color, parent);
    }
}

pub fn label_created(json: bool, label: &Label) {
    if json {
        json_out(label);
        return;
    }
    println!("Created '{}' (id {})", label.name, label.id);
}

pub fn attachments_list(json: bool, atts: &[Attachment]) {
    if json {
        json_out(atts);
        return;
    }
    println!("{} attachment(s)", atts.len());
    for a in atts {
        let mime = a.mime_type.as_deref().unwrap_or("application/octet-stream");
        let disp = a.disposition.as_deref().unwrap_or("attachment");
        println!("  {} {}  ({} bytes, {mime}, {disp})", a.id, a.name, a.size);
    }
}

/// Report the outcome of a bulk organize action.
pub fn action_result(json: bool, action: &str, ids: &[String]) {
    if json {
        json_out(&json!({ "status": "ok", "action": action, "count": ids.len(), "ids": ids }));
        return;
    }
    println!("{action}: {} item(s)", ids.len());
}

/// Report a successful send (returns message ID).
pub fn sent(json: bool, id: &str) {
    if json {
        json_out(&json!({ "status": "ok", "id": id }));
        return;
    }
    println!("Sent (id {id})");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fmt_time_handles_epoch_and_known_values() {
        // Non-positive timestamps render as a placeholder.
        assert_eq!(fmt_time(0), "-");
        assert_eq!(fmt_time(-5), "-");
        // 1 second past the Unix epoch.
        assert_eq!(fmt_time(1), "1970-01-01 00:00:01 UTC");
        // A well-known timestamp: 1_700_000_000 = 2023-11-14 22:13:20 UTC.
        assert_eq!(fmt_time(1_700_000_000), "2023-11-14 22:13:20 UTC");
    }

    #[test]
    fn civil_from_days_handles_leap_day() {
        // 2024-02-29 00:00:00 UTC = day 19782 since the epoch; the hand-rolled
        // proleptic-Gregorian inverse must land on the leap day exactly.
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(19_782), (2024, 2, 29));
    }

    #[test]
    fn label_name_maps_system_ids_and_passes_through() {
        assert_eq!(label_name("0"), "inbox");
        assert_eq!(label_name("3"), "trash");
        assert_eq!(label_name("10"), "starred");
        // Unknown / custom label ids pass through unchanged.
        assert_eq!(label_name("custom-xyz"), "custom-xyz");
    }
}
