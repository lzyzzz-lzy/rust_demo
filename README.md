# AI文本增强系统

这是一个基于 Rust 和 React 的 AI 文本增强系统，用于处理和分析文档内容。系统采用前后端分离架构，后端使用 Rust 实现高性能的文本处理服务，前端使用 React + TypeScript 构建现代化的用户界面。

## 技术栈

### 后端技术栈
- Rust
- Actix-web (Web框架)
- Tokio (异步运行时)
- Serde (序列化/反序列化)

### 前端技术栈
- React 18
- TypeScript
- Tailwind CSS
- Vite

## 环境要求

### 后端环境
1. Rust环境
   - 访问 https://rustup.rs/ 下载并安装 Rust
   - 运行安装程序后，打开新的终端窗口验证安装：
     ```bash
     rustc --version
     cargo --version
     ```
   - 确保 Rust 工具链安装完整（如果没有会自动安装）：
     ```bash
     rustup component add rustfmt
     rustup component add clippy
     ```

### 前端环境
1. Node.js环境
   - 访问 https://nodejs.org/ 下载并安装 Node.js LTS版本（建议 v18.0.0 或更高版本）
   - 验证安装：
     ```bash
     node --version
     npm --version
     ```

## 项目结构
```
rust_demo/
├── frontend/          # React前端项目
│   ├── src/          # 源代码目录
│   │   ├── components/   # React组件
│   │   ├── App.tsx      # 主应用组件
│   │   └── main.tsx     # 应用入口
│   └── public/       # 静态资源
├── backend/          # Rust后端项目
│   ├── src/         # 源代码目录
│   │   ├── api/     # API接口定义
│   │   ├── handlers/# 请求处理器
│   │   └── models/  # 数据模型
│   └── uploads/     # 文件上传目录
```

## 功能特性
- 文件处理
  - 支持上传 Word 和 TXT 文件
  - 自动文件类型检测和验证
  - 安全的文件存储机制
- 文本分析
  - 文本内容智能分析与归纳
  - 自动生成文档大纲
  - 关键信息提取
- 文本优化
  - 智能文本润色
  - 语法纠错
  - 风格优化建议
- AI 增强
  - 智能续写
  - 上下文相关的建议
  - 风格化评语生成

## 开发说明

### 后端开发
1. 进入后端目录：
   ```bash
   cd backend
   ```

2. 安装依赖并构建项目：
   ```bash
   cargo build
   ```

3. 运行开发服务器：
   ```bash
   cargo run
   ```
   服务器默认运行在 http://localhost:8080

4. 运行测试：
   ```bash
   cargo test
   ```

### 前端开发
1. 进入前端目录：
   ```bash
   cd frontend
   ```

2. 安装依赖：
   ```bash
   npm install
   ```

3. 启动开发服务器：
   ```bash
   npm run dev
   ```
   开发服务器默认运行在 http://localhost:5173

4. 构建生产版本：
   ```bash
   npm run build
   ```

## 配置说明

### 后端配置
- 默认端口：8080
- 文件上传目录：`/backend/uploads`
- 最大文件大小：10MB

### 前端配置
- API 基础URL：在 `.env` 文件中配置
- 开发服务器端口：5173
- 生产构建输出目录：`/frontend/dist`

## 部署说明

### 后端部署
1. 构建发布版本：
   ```bash
   cd backend
   cargo build --release
   ```

2. 运行服务：
   ```bash
   ./target/release/rust_demo
   ```

### 前端部署
1. 构建生产版本：
   ```bash
   cd frontend
   npm run build
   ```

2. 将 `dist` 目录下的文件部署到 Web 服务器

## 注意事项
- 确保后端服务器有足够的磁盘空间用于文件上传
- 定期清理 uploads 目录下的临时文件
- 在生产环境中配置适当的 CORS 策略
- 建议使用 HTTPS 协议保护数据传输安全 