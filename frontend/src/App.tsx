import { useState } from 'react'
import { Layout, Typography } from 'antd'
import FileUpload from './components/FileUpload'
import TextProcessor from './components/TextProcessor'
import type { FileInfo } from './types'

const { Header, Content } = Layout
const { Title } = Typography

function App() {
  const [uploadedFile, setUploadedFile] = useState<(FileInfo & { filepath?: string }) | null>(null)

  return (
    <Layout className="min-h-screen">
      <Header className="bg-white shadow">
        <Title level={3} className="text-center py-4">AI文本增强系统</Title>
      </Header>
      <Content className="p-6">
        <div className="max-w-4xl mx-auto">
          <FileUpload onFileUploaded={setUploadedFile} />
          {uploadedFile && <TextProcessor file={uploadedFile} />}
        </div>
      </Content>
    </Layout>
  )
}

export default App 