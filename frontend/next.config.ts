/** @type {import('next').NextConfig} */
const nextConfig = {
  output: 'export',  // 使 Next.js 生成静态文件
  images: {
    unoptimized: true,
  },
  // 如果在开发过程中出现路径问题，可以添加：
  assetPrefix: './',
  trailingSlash: true,
}

module.exports = nextConfig