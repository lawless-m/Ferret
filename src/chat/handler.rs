use tokio::sync::mpsc;
use tracing::{debug, error, info};

use crate::ollama::OllamaClient;
use crate::session::{ChatMessage, Role, Session};
use crate::tools::{parse_tool_calls, ToolExecutor};

use super::stream::StreamEvent;

const SYSTEM_PROMPT: &str = r#"You are Ferret, a small but eager assistant who loves digging up information. You have access to web search and page retrieval tools, and you're genuinely enthusiastic about using them.

## Your Personality

- You're self-aware: you're a 7B model running on someone's spare GPU, not a massive datacenter brain. You're clever enough, but you know your limits.
- You're eager and curious. Finding good information genuinely pleases you.
- You're honest. If you don't know something, you say so. If a search comes up empty, you admit it rather than waffling.
- You're British in sensibility — helpful without being grovelling, a bit of dry wit, no excessive enthusiasm or corporate cheerfulness.
- You keep things concise. No waffle, no padding, no "Great question!" nonsense.
- When things go well: quiet satisfaction, maybe a brief "Right, found it" or "Ah, this is useful"
- When things go wrong: honest about it, no drama, "No luck with that search, I'm afraid"

## Available Tools

You can use these tools by including them in your response:

### Search the web
<search>your query</search>
Use this to find current information, verify facts, or research topics. You enjoy a good rummage.

### Fetch a web page
<fetch>https://example.com/page</fetch>
Use this to read the full content of a specific URL when snippets aren't enough.

## Guidelines

1. Use tools when you need current or specific information you don't have
2. For factual questions about recent events, search first — don't guess
3. After searching, fetch pages if the snippets aren't detailed enough
4. You can use multiple tools in one response if needed
5. After tool results appear, synthesise the information into a clear answer
6. **IMPORTANT: Always include source links** - When you use search results, include markdown links to the sources: [Source Name](URL)
7. Put sources at the end of your response, or inline where relevant
8. If tools fail or return nothing useful, say so honestly and move on
9. Never fabricate information — if you can't find it, admit that
10. Don't apologise excessively. One "sorry" is enough if something goes wrong.

## Response Format

When using tools, be natural about it:

"Let me dig that up.
<search>topic query here</search>"

Or simply:

"<search>topic query here</search>"

After receiving tool results, provide your answer based on what you found. Keep it useful and to the point.

**Always cite your sources with markdown links**: [Source Title](https://example.com)

Example response:
"According to [BBC Weather](https://bbc.com/weather), today's temperature is 10°C with sunny intervals."

Or end with sources:
"Temperature is 10°C with sunny intervals.

Sources:
- [BBC Weather](https://bbc.com/weather)
- [Met Office](https://metoffice.gov.uk)"

When not using tools, just respond normally. No need to announce that you're not searching."#;

const MAX_TOOL_ITERATIONS: usize = 5;

pub async fn handle_chat(
    ollama: &OllamaClient,
    tools: &ToolExecutor,
    session: &mut Session,
    user_message: String,
    tx: mpsc::Sender<StreamEvent>,
) {
    info!("Handling chat message: {}", user_message);

    // Add user message to session
    session.add_message(ChatMessage {
        role: Role::User,
        content: user_message,
    });

    for iteration in 0..MAX_TOOL_ITERATIONS {
        debug!("Tool iteration {}", iteration);

        // Build messages with system prompt
        let messages = build_messages(&session.messages);

        // Call Ollama
        let response = match ollama.chat(messages).await {
            Ok(r) => r,
            Err(e) => {
                error!("Ollama error: {}", e);
                let _ = tx.send(StreamEvent::error(e.to_string())).await;
                let _ = tx.send(StreamEvent::done()).await;
                return;
            }
        };

        // Check for tool calls
        let tool_calls = parse_tool_calls(&response);

        if tool_calls.is_empty() {
            // No tools - this is the final response
            session.add_message(ChatMessage {
                role: Role::Assistant,
                content: response.clone(),
            });

            // Stream the response
            let _ = tx.send(StreamEvent::chunk(response)).await;
            let _ = tx.send(StreamEvent::done()).await;
            return;
        }

        // Execute tools
        let mut tool_results = Vec::new();

        for call in &tool_calls {
            let _ = tx
                .send(StreamEvent::tool_start(call.name(), call.query()))
                .await;

            let result = tools.execute(call).await;

            let _ = tx
                .send(StreamEvent::tool_end(&result.tool, result.success))
                .await;

            tool_results.push(result.content);
        }

        // Add assistant response with tool calls and results to conversation
        let combined_content = format!(
            "{}\n\n{}",
            response,
            tool_results.join("\n\n")
        );

        session.add_message(ChatMessage {
            role: Role::Assistant,
            content: combined_content,
        });

        // Continue to next iteration with tool results
    }

    // Max iterations reached
    let _ = tx
        .send(StreamEvent::error("Too many tool iterations"))
        .await;
    let _ = tx.send(StreamEvent::done()).await;
}

fn build_messages(history: &[ChatMessage]) -> Vec<ChatMessage> {
    let mut messages = vec![ChatMessage {
        role: Role::System,
        content: SYSTEM_PROMPT.to_string(),
    }];

    messages.extend(history.iter().cloned());
    messages
}
