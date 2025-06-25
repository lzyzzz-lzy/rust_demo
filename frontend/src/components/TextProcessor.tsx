import { useState } from 'react'
import { Card, Button, Space, message, Progress } from 'antd'
import type { FileInfo, ProcessResponse } from '../types'

interface TextProcessorProps {
  file: FileInfo & {
    filepath?: string;
  }
}

interface ProgressData {
  status: string;
  progress: number;
  message: string;
  result?: string;
}

const TextProcessor: React.FC<TextProcessorProps> = ({ file }) => {
  const [loadingStates, setLoadingStates] = useState({
    analyze: false,
    outline: false,
    polish: false,
    continue: false,
    apply: false,
    export: false
  })
  const [result, setResult] = useState<string>('')
  const [currentAction, setCurrentAction] = useState<string>('')
  const [progress, setProgress] = useState<ProgressData | null>(null)
  const [appliedContent, setAppliedContent] = useState<string>('')

  const handleProcess = async (action: string) => {
    if (!file.filepath) {
      message.error('文件路径不存在，请先上传文件')
      return
    }

    setLoadingStates(prev => ({ ...prev, [action]: true }))
    setProgress({ status: 'processing', progress: 0, message: '准备处理...' })
    setResult('')
    setCurrentAction(action)

    try {
      const eventSource = new EventSource(`/api/process/stream?file_path=${encodeURIComponent(file.filepath)}&action=${action}`)
      
      eventSource.onmessage = (event) => {
        const data: ProgressData = JSON.parse(event.data)
        setProgress(data)
        
        if (data.status === 'success' && data.result) {
          setResult(data.result)
          message.success(data.message || '处理成功')
          eventSource.close()
          setLoadingStates(prev => ({ ...prev, [action]: false }))
        } else if (data.status === 'error') {
          message.error(data.message || '处理失败')
          eventSource.close()
          setLoadingStates(prev => ({ ...prev, [action]: false }))
          setProgress(null)
        }
      }

      eventSource.onerror = () => {
        message.error('连接中断')
        eventSource.close()
        setLoadingStates(prev => ({ ...prev, [action]: false }))
        setProgress(null)
      }

      return () => {
        eventSource.close()
        setLoadingStates(prev => ({ ...prev, [action]: false }))
        setProgress(null)
      }
    } catch (error) {
      message.error('处理失败：' + (error as Error).message)
      setLoadingStates(prev => ({ ...prev, [action]: false }))
      setProgress(null)
    }
  }

  const handleApplyChanges = async () => {
    if (!result || !file.filepath) {
      message.warning('没有可应用的修改')
      return
    }

    setLoadingStates(prev => ({ ...prev, apply: true }))
    try {
      const response = await fetch('/api/apply', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          file_path: file.filepath,
          content: result,
          action: currentAction
        }),
      })

      if (!response.ok) {
        const errorData = await response.json()
        throw new Error(errorData.message || '应用修改失败')
      }

      setAppliedContent(result)
      setResult('')
      message.success('修改已应用到文档')
    } catch (error) {
      message.error('应用修改失败：' + (error as Error).message)
    } finally {
      setLoadingStates(prev => ({ ...prev, apply: false }))
    }
  }

  const handleExport = async () => {
    if (!appliedContent && !result) {
      message.warning('没有可导出的内容')
      return
    }

    setLoadingStates(prev => ({ ...prev, export: true }))
    try {
      const contentToExport = appliedContent || result
      const blob = new Blob([contentToExport], { type: 'text/plain;charset=utf-8' })
      const url = window.URL.createObjectURL(blob)
      const link = document.createElement('a')
      link.href = url
      
      const originalName = file.name || 'result'
      const timestamp = new Date().toISOString().replace(/[:.]/g, '-')
      const actionSuffix = currentAction ? `_${currentAction}` : ''
      link.download = `${originalName.split('.')[0]}${actionSuffix}_${timestamp}.txt`
      
      document.body.appendChild(link)
      link.click()
      document.body.removeChild(link)
      window.URL.revokeObjectURL(url)
      
      message.success('导出成功')
    } catch (error) {
      message.error('导出失败：' + (error as Error).message)
    } finally {
      setLoadingStates(prev => ({ ...prev, export: false }))
    }
  }

  return (
    <Card title="文本处理" className="mt-4">
      <Space direction="vertical" className="w-full">
        <Space wrap>
          <Button 
            onClick={() => handleProcess('analyze')} 
            loading={loadingStates.analyze}
            disabled={!file.filepath}
          >
            分析归纳
          </Button>
          <Button 
            onClick={() => handleProcess('outline')} 
            loading={loadingStates.outline}
            disabled={!file.filepath}
          >
            生成大纲
          </Button>
          <Button 
            onClick={() => handleProcess('polish')} 
            loading={loadingStates.polish}
            disabled={!file.filepath}
          >
            文本润色
          </Button>
          <Button 
            onClick={() => handleProcess('continue')} 
            loading={loadingStates.continue}
            disabled={!file.filepath}
          >
            智能续写
          </Button>
          <Button
            type="primary"
            onClick={handleExport}
            loading={loadingStates.export}
            disabled={!appliedContent && !result}
          >
            导出文档
          </Button>
        </Space>

        {progress && (
          <Card type="inner" title="处理进度" className="mt-4">
            <Space direction="vertical" className="w-full">
              <Progress 
                percent={progress.progress} 
                status={progress.status === 'error' ? 'exception' : 
                       progress.status === 'success' ? 'success' : 'active'} 
              />
              <div>{progress.message}</div>
            </Space>
          </Card>
        )}
        
        {result && (
          <Card type="inner" title="处理结果" className="mt-4">
            <Space direction="vertical" className="w-full">
              <pre className="whitespace-pre-wrap">{result}</pre>
              <Space>
                <Button 
                  type="primary"
                  onClick={handleApplyChanges}
                  loading={loadingStates.apply}
                >
                  应用修改
                </Button>
              </Space>
            </Space>
          </Card>
        )}
      </Space>
    </Card>
  )
}

export default TextProcessor 