import type React from "react"
import type { Metadata } from "next"
import { GeistSans } from "geist/font/sans"
import { GeistMono } from "geist/font/mono"
import "./globals.css"
import { Toaster } from "@/components/ui/toaster"

export const metadata: Metadata = {
  title: "G3.GS Short Link",
  description: "Transform your long URLs into short, shareable links",
  generator: "v0.app",
  // 添加静态导出相关的元数据
  robots: {
    index: true,
    follow: true,
  },
}

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode
}>) {
  return (
    <html lang="en">
      <head>
        <style>{`
html {
  font-family: ${GeistSans.style.fontFamily};
  --font-sans: ${GeistSans.variable};
  --font-mono: ${GeistMono.variable};
}
        `}</style>
      <link rel="icon" href="/favicon.ico?v=2" />
      </head>
      <body className="min-h-screen flex flex-col">
        <main className="flex-1">
          {children}
        </main>
        <Toaster />
      </body>
    </html>
  )
}
