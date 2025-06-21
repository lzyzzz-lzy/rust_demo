import { useState } from 'react'
import { Upload, message, Spin } from 'antd'
import { InboxOutlined } from '@ant-design/icons'
import type { UploadProps } from 'antd'
import type { FileInfo, UploadResponse } from '../types'

const { Dragger } = Upload

interface FileUploadProps {
  onFileUploaded: (file: FileInfo & { filepath?: string }) => void
}

const FileUpload: React.FC<FileUploadProps> = ({ onFileUploaded }) => {
  const [uploading, setUploading] = useState(false)

  const props: UploadProps = {
    name: 'file',
    multiple: false,
    action: '/api/upload',
    accept: '.txt,.doc,.docx',
    beforeUpload: (file) => {
      const isValidSize = file.size / 1024 / 1024 < 5
      if (!isValidSize) {
        message.error('文件大小不能超过5MB!')
        return false
      }
      return true
    },
    onChange(info) {
      const { status } = info.file
      
      if (status === 'uploading') {
        setUploading(true)
        return
      }

      setUploading(false)
      
      if (status === 'done') {
        const response = info.file.response as UploadResponse
        if (response && response.status === 'success' && response.filepath) {
          const fileWithPath = {
            ...info.file.originFileObj,
            filepath: response.filepath
          } as FileInfo & { filepath: string }
          message.success(response.message || `${info.file.name} 文件上传成功`)
          onFileUploaded(fileWithPath)
        } else {
          console.error('Upload failed:', response)
          message.error(response.message || '上传失败')
        }
      } else if (status === 'error') {
        console.error('Upload error:', info.file.error)
        message.error(`${info.file.name} 文件上传失败`)
      }
    },
    onDrop(e) {
      console.log('Dropped files', e.dataTransfer.files)
    },
  }

  return (
    <div className="mb-8 relative">
      <Spin spinning={uploading} tip="上传中...">
        <Dragger {...props}>
          <p className="ant-upload-drag-icon">
            <InboxOutlined />
          </p>
          <p className="ant-upload-text">点击或拖拽文件到此区域上传</p>
          <p className="ant-upload-hint">
            支持单个文件上传，可上传 .txt, .doc, .docx 格式，文件大小不超过5MB
          </p>
        </Dragger>
      </Spin>
    </div>
  )
}

export default FileUpload 