use anyhow::Result;
use simple_chat_client::{
    chat::ChatSession,
    client::OpenAIClient,
    config::Config,
    tool::{get_mcp_tools, Tool, ToolSet},
};
use std::sync::Arc;

//default config path
const DEFAULT_CONFIG_PATH: &str = "/etc/simple-chat-client/config.toml";

#[tokio::main]
async fn main() -> Result<()> {
    // load config
    let config = Config::load(DEFAULT_CONFIG_PATH).await?;

    // create openai client
    let api_key = config
        .openai_key
        .clone()
        .unwrap_or_else(|| std::env::var("OPENAI_API_KEY").expect("need set api key"));
    let url = config.chat_url.clone();
    println!("url is {:?}", url);
    let openai_client = Arc::new(OpenAIClient::new(api_key, url));

    // create tool set
    let mut tool_set = ToolSet::new();

    // load mcp
    if let Some(_) = &config.mcp {
        let mcp_clients = config.create_mcp_clients().await?;

        for (name, client) in mcp_clients {
            println!("正在加载MCP工具: {}", name);
            let server = client.peer().clone();
            let tools = get_mcp_tools(server).await?;

            for tool in tools {
                println!("添加工具: {}", tool.name());
                tool_set.add_tool(tool);
            }
        }
    }

    // create chat session
    let mut session = ChatSession::new(
        openai_client,
        tool_set,
        config
            .model_name
            .unwrap_or_else(|| "nebulacoder-v6.0".to_string()),
    );

    // build system prompt with tool info
    let mut system_prompt =
        "你是一个助手，可以帮助用户完成各种任务。你有以下工具可以使用：\n".to_string();

    // add tool info to system prompt
    for tool in session.get_tools() {
        system_prompt.push_str(&format!(
            "\n工具名称: {}\n描述: {}\n参数格式: {}\n",
            tool.name(),
            tool.description(),
            serde_json::to_string_pretty(&tool.parameters()).unwrap_or_default()
        ));
    }

    // add tool call format guidance
    system_prompt.push_str(
        "\n如果需要调用工具，请使用以下格式：\n\
        Tool: <工具名>\n\
        Inputs: <输入参数>\n",
    );

    // add system prompt
    session.add_system_prompt(system_prompt);

    // start chat
    session.chat().await?;

    Ok(())
}
