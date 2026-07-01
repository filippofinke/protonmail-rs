//! `read_conversation` — read a whole conversation thread.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, ErrorData};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::common::*;
use crate::server::ProtonMail;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReadConversationParams {
    /// Conversation id, a message id within it, or free text identifying a message.
    pub reference: String,
}

#[tool_router(router = read_conversation_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(
        name = "read_conversation",
        description = "Read a whole conversation thread (all decrypted messages). Accepts a conversation id, a message id, or free text."
    )]
    pub async fn read_conversation(
        &self,
        Parameters(p): Parameters<ReadConversationParams>,
    ) -> Result<Out, ErrorData> {
        let mut guard = self.state.client.lock().await;
        self.ensure(&mut guard).await?;
        let client = guard.as_ref().expect("client present");

        let id = self.resolve(client, &p.reference).await?;

        // The resolved id may be a conversation id or a message id. Try it as a
        // conversation first; on a not-found result, treat it as a message id
        // and read the conversation it belongs to.
        let (conv, msgs) = match client.read_conversation(&id).await {
            Ok(x) => x,
            Err(e) if e.exit_code() == 3 => {
                let m = client
                    .read_message(&id)
                    .await
                    .map_err(|e| self.map_err(e))?;
                client
                    .read_conversation(&m.meta.conversation_id)
                    .await
                    .map_err(|e| self.map_err(e))?
            }
            Err(e) => return Err(self.map_err(e)),
        };

        let messages: Vec<Value> = msgs
            .iter()
            .map(|m| {
                Ok(json!({
                    "id": m.meta.id,
                    "meta": to_value(&m.meta)?,
                    "verdict": to_value(&m.verdict)?,
                    "mime_type": m.mime_type,
                    "body": m.body,
                    "warning": signature_warning(m.verdict),
                }))
            })
            .collect::<Result<_, ErrorData>>()?;

        Ok(obj(json!({
            "conversation": to_value(&conv)?,
            "messages": messages,
        })))
    }
}
