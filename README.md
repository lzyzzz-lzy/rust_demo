# AI文本增强系统

这是一个基于Rust和React的AI文本增强系统，用于处理和分析文档内容。

## 环境要求

### 后端环境
1. Rust环境
   - 访问 https://rustup.rs/ 下载并安装 Rust
   - 运行安装程序后，打开新的终端窗口验证安装：
     ```bash
     rustc --version
     cargo --version
     ```

### 前端环境
1. Node.js环境
   - 访问 https://nodejs.org/ 下载并安装 Node.js LTS版本
   - 验证安装：
     ```bash
     node --version
     npm --version
     ```

## 项目结构
```
rust_demo/
├── frontend/     # React前端项目
└── backend/      # Rust后端项目
```

## 功能特性
- 支持上传Word和TXT文件
- 文本内容分析与归纳
- 自动生成文档大纲
- 文本润色与优化
- 智能续写
- 风格化评语生成

## 开发说明
1. 后端开发
   ```bash
   cd backend
   cargo build
   cargo run
   ```

2. 前端开发
   ```bash
   cd frontend
   npm install
   npm start
   ``` 