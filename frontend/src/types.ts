// 文件上传响应
export interface UploadResponse {
    status: string;
    message: string;
    filepath?: string;
}

// 文本处理请求
export interface ProcessRequest {
    file_path: string;
    action: 'analyze' | 'outline' | 'polish' | 'continue';
}

// 文本处理响应
export interface ProcessResponse {
    status: string;
    message: string;
    result?: string;
}

// 通用错误响应
export interface ErrorResponse {
    status: string;
    message: string;
}

// 文件类型（用于上传组件）
export interface FileInfo {
    name: string;
    size: number;
    type: string;
    lastModified: number;
} 