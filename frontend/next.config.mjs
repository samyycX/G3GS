/** @type {import('next').NextConfig} */
const nextConfig = {
  eslint: {
    ignoreDuringBuilds: true,
  },
  typescript: {
    ignoreBuildErrors: true,
  },
  images: {
    unoptimized: true,
  },
  // 添加静态导出配置
  output: 'export',
  // 静态导出的基础路径，如果部署到子目录需要设置
  // basePath: '/your-subdirectory',
  // 跳过trailingSlash检查，避免导出时的警告
  trailingSlash: true,
}

export default nextConfig
