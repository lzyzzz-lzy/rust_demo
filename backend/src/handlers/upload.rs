use actix_web::{Error, HttpResponse};
use std::fs;
use std::path::Path;
use actix_multipart::Multipart;
use futures::{StreamExt, TryStreamExt};
use uuid::Uuid;
use sanitize_filename::sanitize;
use std::io::Write;
use std::path::PathBuf;

pub async fn upload_file(mut payload: Multipart) -> Result<HttpResponse, Error> {
    // 创建uploads目录（如果不存在）
    let upload_dir = PathBuf::from("uploads");
    if let Err(e) = fs::create_dir_all(&upload_dir) {
        return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "status": "error",
            "message": format!("无法创建上传目录: {}", e)
        })));
    }

    let mut saved_file = None;

    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition();
        
        let filename = content_disposition
            .get_filename()
            .map_or_else(|| Uuid::new_v4().to_string(), |f| sanitize(f));
            
        let filepath = upload_dir.join(&filename);
        
        // 创建文件
        let mut file = match fs::File::create(&filepath) {
            Ok(file) => file,
            Err(e) => {
                return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "status": "error",
                    "message": format!("无法创建文件: {}", e)
                })));
            }
        };
        
        // 写入文件内容
        while let Some(chunk) = field.next().await {
            let data = chunk?;
            if let Err(e) = file.write_all(&data) {
                return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "status": "error",
                    "message": format!("写入文件失败: {}", e)
                })));
            }
        }

        saved_file = Some(filepath);
    }
    
    if let Some(filepath) = saved_file {
        // 将路径转换为字符串，并使用正斜杠
        let filepath_str = filepath.to_string_lossy().replace('\\', "/");
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "message": "文件上传成功",
            "filepath": filepath_str
        })))
    } else {
        Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "status": "error",
            "message": "没有收到文件"
        })))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::header::{ContentDisposition, DispositionType};
    use actix_web::web::Bytes;
    use futures::stream;
    use std::fs;
    use std::path::Path;

    #[actix_rt::test]
    async fn test_upload_file_success() {
        // 准备测试目录
        let test_dir = "test_uploads";
        let _ = fs::create_dir_all(test_dir);
        
        // 创建测试文件内容
        let content = "Hello, World!";
        let filename = "test.txt";
        
        // 创建multipart payload
        let content_disposition = ContentDisposition {
            disposition: DispositionType::FormData,
            parameters: vec![],
        };
        
        let payload = Multipart::new(
            &content_disposition,
            stream::once(async move { Ok(Bytes::from(content)) }),
        );
        
        // 执行上传
        let response = upload_file(payload).await.unwrap();
        
        // 验证响应
        assert!(response.status().is_success());
        
        // 清理测试文件
        let _ = fs::remove_dir_all(test_dir);
    }

    #[actix_rt::test]
    async fn test_upload_file_no_content() {
        // 创建空的multipart payload
        let content_disposition = ContentDisposition {
            disposition: DispositionType::FormData,
            parameters: vec![],
        };
        
        let payload = Multipart::new(
            &content_disposition,
            stream::empty::<Result<Bytes, actix_web::Error>>(),
        );
        
        // 执行上传
        let response = upload_file(payload).await.unwrap();
        
        // 验证响应
        assert!(response.status().is_client_error());
    }
} 