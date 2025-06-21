use actix_web::{web, Error, HttpResponse};
use serde::{Deserialize, Serialize};
use crate::api::ai::{Message, send_chat_request};
use std::fs;
use std::path::Path;
use actix_web::web::Bytes;
use std::time::Duration;
use tokio::time::sleep;
use log::{info, error, warn};

#[derive(Debug, Deserialize)]
pub struct ProcessRequest {
    pub file_path: String,
    pub action: String,  // "analyze", "outline", "polish", "continue"
}

#[derive(Debug, Serialize)]
pub struct ProcessResponse {
    pub status: String,
    pub message: String,
    pub result: Option<String>,
}

#[derive(Debug, Serialize)]
struct ProgressUpdate {
    pub status: String,
    pub progress: i32,
    pub message: String,
    pub result: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ApplyRequest {
    pub file_path: String,
    pub content: String,
    pub action: String,
}

fn get_prompt_and_format<'a>(action: &str, content: &'a str) -> (String, &'a str) {
    match action {
        "analyze" => (
            format!(
                "请分析以下文本的主要内容、写作风格和关键观点。\n\n
                要求：\n
                1. 分点列出，每点都要有具体论述\n
                2. 不要包含'这段文字'、'这篇文章'等指代词\n
                3. 不要出现'总的来说'、'综上所述'等总结性词语\n
                4. 直接给出分析结果，不要有开场白\n\n
                文本内容：\n{}", 
                content
            ),
            "analysis"
        ),
        "outline" => (
            format!(
                "请为以下文本生成一个详细的大纲。\n\n
                要求：\n
                1. 使用标准的大纲格式（如：I、II、III...）\n
                2. 每个大点下应有2-3个小点\n
                3. 保持层级清晰\n
                4. 直接给出大纲，不要有任何说明性文字\n\n
                文本内容：\n{}", 
                content
            ),
            "outline"
        ),
        "polish" => (
            format!(
                "请润色以下文本，使其更加流畅、专业，但保持原意不变。\n\n\
                要求：\n\
                1. 如果文本中包含\"文章大纲:\"开头的部分，请将该部分完整保留，不要做任何修改\n\
                2. 如果看到类似\"（此处大纲内容保留不变）\"的注释，请直接删除这个注释\n\
                3. 保持原文的段落结构和格式\n\
                4. 不要添加新的内容\n\
                5. 不要改变原文的观点和论述\n\
                6. 直接返回完整的文本，不要有任何说明性文字\n\n\
                文本内容：\n{}", 
                content
            ),
            "polish"
        ),
        "continue" => (
            format!(
                "请基于以下文本的风格和内容续写。\n\n\
                要求：\n\
                1. 保持原文的写作风格和语气\n\
                2. 续写内容要自然衔接，从最后一段开始续写\n\
                3. 续写长度不超过原文的1/3\n\
                4. 直接给出续写内容，不要有任何说明性文字\n\n\
                文本内容：\n{}", 
                content
            ),
            "continue"
        ),
        _ => (String::new(), "unknown"),
    }
}

fn process_ai_response(response: String, format_type: &str) -> String {
    match format_type {
        "analysis" => {
            // 移除可能的开场白和结束语
            let content = response
                .lines()
                .filter(|line| {
                    !line.contains("这段文字") && 
                    !line.contains("这篇文章") && 
                    !line.contains("总的来说") && 
                    !line.contains("综上所述")
                })
                .collect::<Vec<&str>>()
                .join("\n");
            content
        },
        "outline" => {
            // 只保留大纲格式的内容
            let content = response
                .lines()
                .filter(|line| {
                    line.trim().starts_with(|c| {
                        matches!(c, 'I' | 'V' | 'X' | 'i' | '1' | '2' | '3' | '-' | '•')
                    }) || line.trim().is_empty()
                })
                .collect::<Vec<&str>>()
                .join("\n");
            content
        },
        "polish" => {
            // 移除可能的说明性文字、标记和注释
            response
                .lines()
                .filter(|line| {
                    !line.contains("润色后的文本") && 
                    !line.contains("修改建议") &&
                    !line.contains("修改后的文本") &&
                    !line.contains("（此处大纲内容保留不变）") &&  // 添加对注释的过滤
                    !line.contains("(此处大纲内容保留不变)") &&    // 同时处理中英文括号
                    !line.trim().is_empty()  // 移除空行
                })
                .collect::<Vec<&str>>()
                .join("\n")
        },
        "continue" => {
            // 移除可能的说明性文字
            response
                .lines()
                .filter(|line| {
                    !line.contains("续写内容") && 
                    !line.contains("接下来") &&
                    !line.contains("继续写道")
                })
                .collect::<Vec<&str>>()
                .join("\n")
        },
        _ => response,
    }
}

pub async fn process_text_stream(
    request: web::Query<ProcessRequest>,
) -> Result<HttpResponse, Error> {
    info!("开始处理文本请求: action={}", request.action);
    
    // 设置SSE响应头
    let response = HttpResponse::Ok()
        .append_header(("Content-Type", "text/event-stream"))
        .append_header(("Cache-Control", "no-cache"))
        .append_header(("Connection", "keep-alive"))
        .streaming(async_stream::stream! {
            // 读取文件内容
            let content = match fs::read_to_string(&request.file_path) {
                Ok(content) => {
                    info!("成功读取文件: {}", request.file_path);
                    content
                },
                Err(e) => {
                    error!("读取文件失败: {}", e);
                    let error = ProgressUpdate {
                        status: "error".to_string(),
                        progress: 0,
                        message: format!("无法读取文件: {}", e),
                        result: None,
                    };
                    yield Ok::<Bytes, Error>(Bytes::from(format!("data: {}\n\n", serde_json::to_string(&error).unwrap())));
                    return;
                }
            };

            // 发送开始处理的进度
            let start = ProgressUpdate {
                status: "processing".to_string(),
                progress: 10,
                message: "开始处理文本...".to_string(),
                result: None,
            };
            yield Ok::<Bytes, Error>(Bytes::from(format!("data: {}\n\n", serde_json::to_string(&start).unwrap())));
            sleep(Duration::from_millis(500)).await;

            // 获取提示词和格式类型
            let (prompt, format_type) = get_prompt_and_format(&request.action, &content);
            if prompt.is_empty() {
                warn!("不支持的操作类型: {}", request.action);
                let error = ProgressUpdate {
                    status: "error".to_string(),
                    progress: 0,
                    message: "不支持的操作类型".to_string(),
                    result: None,
                };
                yield Ok::<Bytes, Error>(Bytes::from(format!("data: {}\n\n", serde_json::to_string(&error).unwrap())));
                return;
            }

            // 发送处理中的进度
            let processing = ProgressUpdate {
                status: "processing".to_string(),
                progress: 30,
                message: "正在处理文本...".to_string(),
                result: None,
            };
            yield Ok::<Bytes, Error>(Bytes::from(format!("data: {}\n\n", serde_json::to_string(&processing).unwrap())));

            // 构建消息
            let messages = vec![
                Message {
                    role: "user".to_string(),
                    content: prompt,
                }
            ];

            info!("开始调用AI服务...");
            // 发送请求到AI服务
            match send_chat_request(messages).await {
                Ok(result) => {
                    info!("AI服务调用成功");
                    // 处理AI响应
                    let processed_result = process_ai_response(result, format_type);

                    // 根据操作类型组合结果
                    let final_result = match request.action.as_str() {
                        "analyze" => format!("文本分析：\n{}\n\n---\n\n{}", processed_result, content),
                        "outline" => format!("文章大纲：\n{}\n\n---\n\n{}", processed_result, content),
                        "polish" => processed_result,
                        "continue" => {
                            // 检查是否已经有续写内容
                            if content.contains("\n---\n") {
                                // 如果有分隔的部分，追加到最后一个部分
                                let parts: Vec<&str> = content.split("\n---\n").collect();
                                let mut new_content = content.to_string();
                                new_content.push_str("\n\n");
                                new_content.push_str(&processed_result);
                                new_content
                            } else {
                                // 如果没有分隔，直接追加到末尾
                                format!("{}\n\n{}", content, processed_result)
                            }
                        },
                        _ => processed_result,
                    };

                    // 发送完成的进度
                    let complete = ProgressUpdate {
                        status: "success".to_string(),
                        progress: 100,
                        message: "处理完成".to_string(),
                        result: Some(final_result),
                    };
                    yield Ok::<Bytes, Error>(Bytes::from(format!("data: {}\n\n", serde_json::to_string(&complete).unwrap())));
                }
                Err(e) => {
                    error!("AI服务调用失败: {}", e);
                    let error = ProgressUpdate {
                        status: "error".to_string(),
                        progress: 0,
                        message: format!("AI处理失败: {}", e),
                        result: None,
                    };
                    yield Ok::<Bytes, Error>(Bytes::from(format!("data: {}\n\n", serde_json::to_string(&error).unwrap())));
                }
            }
        });

    Ok(response)
}

// 在 apply_changes 函数之前添加这些辅助函数
fn update_section(content: &str, section_type: &str, new_content: &str) -> String {
    let marker = match section_type {
        "分析" => "文本分析：",
        "大纲" => "文章大纲：",
        "续写" => "",  // 移除续写的标记，使其无感
        _ => return content.to_string(),
    };

    let parts: Vec<&str> = content.split("\n---\n").collect();
    
    // 检查第一部分是否包含指定的标记
    if parts[0].starts_with(marker) {
        // 更新已存在的部分
        let mut result = if section_type == "续写" {
            // 对于续写，直接追加到最后一个部分
            let mut content = parts[0..parts.len()-1].join("\n---\n");
            content.push_str("\n\n");
            content.push_str(&new_content);
            content
        } else {
            // 对于其他类型，更新第一部分
            let mut result = format!("{}\n{}", marker, new_content);
            if parts.len() > 1 {
                result.push_str("\n\n---\n\n");
                result.push_str(&parts[1..].join("\n---\n"));
            }
            result
        };
        result
    } else {
        // 添加新的部分
        match section_type {
            "续写" => format!("{}\n\n{}", content, new_content),
            _ => format!("{}\n{}\n\n---\n\n{}", marker, new_content, content),
        }
    }
}

pub async fn apply_changes(
    request: web::Json<ApplyRequest>,
) -> Result<HttpResponse, Error> {
    info!("开始应用修改: action={}", request.action);
    
    // 验证文件路径
    let path = Path::new(&request.file_path);
    if !path.exists() {
        warn!("文件不存在: {}", request.file_path);
        return Ok(HttpResponse::BadRequest().json(ProcessResponse {
            status: "error".to_string(),
            message: "文件不存在".to_string(),
            result: None,
        }));
    }

    // 读取原文件内容
    let original_content = match fs::read_to_string(&request.file_path) {
        Ok(content) => content,
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(ProcessResponse {
                status: "error".to_string(),
                message: format!("读取文件失败: {}", e),
                result: None,
            }));
        }
    };

    // 创建备份
    let backup_path = format!("{}.bak", request.file_path);
    if let Err(e) = fs::copy(&request.file_path, &backup_path) {
        return Ok(HttpResponse::InternalServerError().json(ProcessResponse {
            status: "error".to_string(),
            message: format!("创建备份失败: {}", e),
            result: None,
        }));
    }

    // 根据不同的操作类型处理内容
    let new_content = match request.action.as_str() {
        "outline" => update_section(&original_content, "大纲", &request.content),
        "polish" => {
            // 处理润色内容，移除注释
            request.content
                .lines()
                .filter(|line| {
                    !line.contains("（此处大纲内容保留不变）") &&
                    !line.contains("(此处大纲内容保留不变)") &&
                    !line.trim().is_empty()
                })
                .collect::<Vec<&str>>()
                .join("\n")
        },
        "continue" => {
            // 检查是否已经有续写内容
            if original_content.contains("\n---\n") {
                // 如果有分隔的部分，追加到最后一个部分
                let parts: Vec<&str> = original_content.split("\n---\n").collect();
                let mut new_content = parts[0..parts.len()-1].join("\n---\n");
                new_content.push_str("\n\n");
                new_content.push_str(&request.content);
                new_content
            } else {
                // 如果没有分隔，直接追加到末尾
                format!("{}\n\n{}", original_content, request.content)
            }
        },
        _ => {
            return Ok(HttpResponse::BadRequest().json(ProcessResponse {
                status: "error".to_string(),
                message: "不支持的操作类型".to_string(),
                result: None,
            }));
        }
    };

    // 写入新内容
    match fs::write(&request.file_path, new_content) {
        Ok(_) => Ok(HttpResponse::Ok().json(ProcessResponse {
            status: "success".to_string(),
            message: "修改已应用".to_string(),
            result: None,
        })),
        Err(e) => {
            // 如果写入失败，尝试恢复备份
            if let Err(backup_err) = fs::copy(&backup_path, &request.file_path) {
                return Ok(HttpResponse::InternalServerError().json(ProcessResponse {
                    status: "error".to_string(),
                    message: format!("写入失败且恢复备份失败: {}, {}", e, backup_err),
                    result: None,
                }));
            }
            Ok(HttpResponse::InternalServerError().json(ProcessResponse {
                status: "error".to_string(),
                message: format!("写入失败，已恢复备份: {}", e),
                result: None,
            }))
        }
    }
} 