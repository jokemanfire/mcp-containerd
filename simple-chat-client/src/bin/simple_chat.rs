use anyhow::Result;
use simple_chat_client::{
    chat::ChatSession,
    client::OpenAIClient,
    config::Config,
    tool::{get_mcp_tools, Tool, ToolSet},
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // 加载配置
    let config = Config::load("config.toml").await?;

    // 创建OpenAI客户端
    let api_key = config.openai_key.clone().unwrap_or_else(|| {
        std::env::var("OPENAI_API_KEY").expect("需要设置OPENAI_API_KEY环境变量或在配置中提供")
    });

    let openai_client = Arc::new(OpenAIClient::new(api_key));

    // 创建工具集
    let mut tool_set = ToolSet::new();

    // 如果配置了MCP，加载MCP工具
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

    // 创建聊天会话
    let mut session = ChatSession::new(openai_client, tool_set, "gpt-3.5-turbo".to_string());

    // 添加系统提示
    session.add_system_prompt(
        "你是一个助手，可以帮助用户完成各种任务。如果需要调用工具，请使用以下格式：\n\
        Tool: <工具名>\n\
        Inputs: <输入参数>",
    );

    // 开始聊天
    session.chat().await?;

    Ok(())
}
