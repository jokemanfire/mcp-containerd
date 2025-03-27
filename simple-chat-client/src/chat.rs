use crate::{
    client::ChatClient,
    model::{CompletionRequest, Message, Tool as ModelTool, ToolCall},
    tool::{Tool as ToolTrait, ToolSet},
};
use anyhow::Result;
use async_trait::async_trait;
use futures::StreamExt;
use serde_json::Value;
use std::io::{self, Write};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;

pub struct ChatSession {
    client: Arc<dyn ChatClient>,
    tool_set: ToolSet,
    model: String,
    messages: Vec<Message>,
}

impl ChatSession {
    pub fn new(client: Arc<dyn ChatClient>, tool_set: ToolSet, model: String) -> Self {
        Self {
            client,
            tool_set,
            model,
            messages: Vec::new(),
        }
    }

    pub fn add_system_prompt(&mut self, prompt: impl ToString) {
        self.messages.push(Message::system(prompt));
    }

    pub async fn chat(&mut self) -> Result<()> {
        println!("欢迎使用简易聊天客户端。输入 'exit' 退出。");

        loop {
            print!("> ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            input = input.trim().to_string();

            if input.is_empty() {
                continue;
            }

            if input == "exit" {
                break;
            }

            self.messages.push(Message::user(&input));

            // 准备工具列表
            let tools = self.tool_set.tools();
            let tool_definitions = if !tools.is_empty() {
                Some(
                    tools
                        .iter()
                        .map(|tool| crate::model::Tool {
                            name: tool.name(),
                            description: tool.description(),
                            parameters: tool.parameters(),
                        })
                        .collect(),
                )
            } else {
                None
            };

            // 创建请求
            let request = CompletionRequest {
                model: self.model.clone(),
                messages: self.messages.clone(),
                temperature: Some(0.7),
                tools: tool_definitions,
            };

            // 发送请求
            let response = self.client.complete(request).await?;

            if let Some(choice) = response.choices.first() {
                println!("AI: {}", choice.message.content);
                self.messages.push(choice.message.clone());

                // 检查消息中是否包含工具调用
                if choice.message.content.contains("Tool:") {
                    let lines: Vec<&str> = choice.message.content.split('\n').collect();

                    // 简单解析工具调用
                    let mut tool_name = None;
                    let mut args_text = Vec::new();
                    let mut parsing_args = false;

                    for line in lines {
                        if line.starts_with("Tool:") {
                            tool_name = line.strip_prefix("Tool:").map(|s| s.trim().to_string());
                            parsing_args = false;
                        } else if line.starts_with("Inputs:") {
                            parsing_args = true;
                        } else if parsing_args {
                            args_text.push(line.trim());
                        }
                    }

                    if let Some(name) = tool_name {
                        if let Some(tool) = self.tool_set.get_tool(&name) {
                            println!("正在调用工具: {}", name);

                            // 简单处理参数
                            let args_str = args_text.join("\n");
                            let args = match serde_json::from_str(&args_str) {
                                Ok(v) => v,
                                Err(_) => {
                                    // 尝试将文本作为字符串处理
                                    serde_json::Value::String(args_str)
                                }
                            };

                            // 调用工具
                            match tool.call(args).await {
                                Ok(result) => {
                                    println!("工具结果: {}", result);

                                    // 添加工具结果到对话
                                    self.messages.push(Message::user(result));
                                }
                                Err(e) => {
                                    println!("工具调用失败: {}", e);
                                    self.messages
                                        .push(Message::user(format!("工具调用失败: {}", e)));
                                }
                            }
                        } else {
                            println!("找不到工具: {}", name);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[async_trait]
impl ToolTrait for ModelTool {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn description(&self) -> String {
        self.description.clone()
    }

    fn parameters(&self) -> Value {
        self.parameters.clone()
    }

    async fn call(&self, _args: Value) -> Result<String> {
        // 这里需要一个真正的实现
        unimplemented!("ModelTool不能直接调用，仅用于传递工具定义")
    }
}
