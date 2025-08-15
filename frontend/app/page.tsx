"use client"

import type React from "react"

import { useState, useEffect, useRef } from "react"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "@/components/ui/dialog"
import { Label } from "@/components/ui/label"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Copy, QrCode, Clock, ExternalLink } from "lucide-react"
import { useToast } from "@/hooks/use-toast"
import {QRCodeSVG} from 'qrcode.react';
import Footer from "@/components/Footer"

interface HistoryItem {
  id: string
  originalUrl: string
  shortUrl: string
  createdAt: string
  expiresAt?: string
  expiration: string
}

export default function Home() {
  const [url, setUrl] = useState("")
  const [expiration, setExpiration] = useState("permanent")
  const [isLoading, setIsLoading] = useState(false)
  const [shortLink, setShortLink] = useState("")
  const [showPopup, setShowPopup] = useState(false)
  const [history, setHistory] = useState<HistoryItem[]>([])
  const qrCanvasRef = useRef<HTMLCanvasElement>(null)
  const { toast } = useToast()

  const expirationOptions = [
    { value: "1h", label: "1 Hour" },
    { value: "12h", label: "12 Hours" },
    { value: "1d", label: "1 Days" },
    { value: "7d", label: "7 Days" },
    { value: "30d", label: "30 Days" },
    { value: "permanent", label: "Permanent" },
  ]

  useEffect(() => {
    // 检查是否在浏览器环境中
    if (typeof window !== 'undefined') {
      const savedHistory = localStorage.getItem("shortlink-history")
      if (savedHistory) {
        setHistory(JSON.parse(savedHistory))
      }
    }
  }, [])

  const isExpired = (item: HistoryItem) => {
    if (!item.expiresAt) return false
    return new Date() > new Date(item.expiresAt)
  }

  const formatTimeRemaining = (expiresAt: string) => {
    const now = new Date()
    const expiry = new Date(expiresAt)
    const diff = expiry.getTime() - now.getTime()
    
    if (diff <= 0) return "Expired"
    
    const seconds = Math.floor(diff / 1000)
    const minutes = Math.floor(seconds / 60)
    const hours = Math.floor(minutes / 60)
    const days = Math.floor(hours / 24)
    
    if (days > 0) {
      return `${days} day${days > 1 ? 's' : ''} ${hours % 24}h ${minutes % 60}m`
    } else if (hours > 0) {
      return `${hours}h ${minutes % 60}m ${seconds % 60}s`
    } else if (minutes > 0) {
      return `${minutes}m ${seconds % 60}s`
    } else {
      return `${seconds}s`
    }
  }

  const generateShortLink = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!url) return

    setIsLoading(true)
    try {
      // 构建请求数据
      const requestData = {
        url: url,
        expires_at: expiration === "permanent"
          ? "1970-01-01T00:00:00.000Z"
          : calculateExpirationDate(expiration)
      }

      const response = await fetch('/api/shorten', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(requestData)
      })

      if (!response.ok) {
        const errorData = await response.json()
        throw new Error(errorData.error || "Failed to generate short link")
      }

      const data = await response.json()
      setShortLink(data.short_url)
      setShowPopup(true)
      
      // 保存到历史记录，使用API返回的数据
      saveToHistoryWithApiResponse(data)
      setUrl("")
    } catch (error) {
      toast({
        title: "Error",
        description: error instanceof Error ? error.message : "Failed to generate short link. Please try again.",
        variant: "destructive",
      })
    } finally {
      setIsLoading(false)
    }
  }

  const calculateExpirationDate = (expiration: string): string => {
    const now = new Date()
    const expirationDate = new Date(now)
    
    switch (expiration) {
      case "1h":
        expirationDate.setHours(expirationDate.getHours() + 1)
        break
      case "12h":
        expirationDate.setHours(expirationDate.getHours() + 12)
        break
      case "1d":
        expirationDate.setDate(expirationDate.getDate() + 1)
        break
      case "7d":
        expirationDate.setDate(expirationDate.getDate() + 7)
        break
      case "30d":
        expirationDate.setDate(expirationDate.getDate() + 30)
        break
      default:
        expirationDate.setHours(expirationDate.getHours() + 1)
    }
    
    return expirationDate.toISOString()
  }

  const saveToHistoryWithApiResponse = (apiResponse: {
    short_url: string
    original_url: string
    created_at: string
    expires_at: string
  }) => {
    const newItem: HistoryItem = {
      id: Date.now().toString(),
      originalUrl: apiResponse.original_url,
      shortUrl: apiResponse.short_url,
      createdAt: apiResponse.created_at,
      expiresAt: apiResponse.expires_at === "1970-01-01T00:00:00Z" ? undefined : apiResponse.expires_at,
      expiration: apiResponse.expires_at === "1970-01-01T00:00:00Z" ? "permanent" : "custom"
    }
    
    const updatedHistory = [newItem, ...history].slice(0, 5)
    setHistory(updatedHistory)
    
    // 检查是否在浏览器环境中
    if (typeof window !== 'undefined') {
      localStorage.setItem("shortlink-history", JSON.stringify(updatedHistory))
    }
  }

  const copyToClipboard = async (text: string) => {
    try {
      // 检查是否在浏览器环境中
      if (typeof window !== 'undefined' && navigator.clipboard) {
        await navigator.clipboard.writeText(text)
        toast({
          title: "Copied!",
          description: "Short link copied to clipboard",
        })
      } else {
        // 降级处理：使用document.execCommand
        const textArea = document.createElement('textarea')
        textArea.value = text
        document.body.appendChild(textArea)
        textArea.select()
        document.execCommand('copy')
        document.body.removeChild(textArea)
        toast({
          title: "Copied!",
          description: "Short link copied to clipboard",
        })
      }
    } catch (error) {
      toast({
        title: "Error",
        description: "Failed to copy to clipboard",
        variant: "destructive",
      })
    }
  }

  const showLinkPopup = (shortUrl: string) => {
    setShortLink(shortUrl)
    setShowPopup(true)
  }

  return (
    <div className="min-h-screen bg-black relative overflow-hidden">
      {/* Enhanced dot pattern background with responsive sizing */}
      <div className="absolute inset-0">
        <div
          className="absolute inset-0"
          style={{
            backgroundImage: `radial-gradient(circle at 1px 1px, rgba(255,255,255,0.3) 1px, transparent 0)`,
            backgroundSize: "20px 20px",
          }}
        />
      </div>

      <div className="absolute inset-0 overflow-hidden pointer-events-none">
        <div className="absolute top-20 left-10 w-40 h-40 border-2 border-white/20 rounded-full animate-pulse"></div>
        <div
          className="absolute top-40 right-20 w-32 h-32 border-2 border-white/20 rounded-lg rotate-45 animate-pulse"
          style={{ animationDelay: "1s" }}
        ></div>
        <div
          className="absolute bottom-32 left-1/4 w-24 h-24 border-2 border-white/20 rounded-full animate-pulse"
          style={{ animationDelay: "2s" }}
        ></div>
        <div
          className="absolute bottom-20 right-1/3 w-28 h-28 border-2 border-white/20 rounded-lg rotate-12 animate-pulse"
          style={{ animationDelay: "0.5s" }}
        ></div>
        <div
          className="absolute top-1/2 left-20 w-16 h-16 border border-white/15 rounded-full animate-pulse"
          style={{ animationDelay: "3s" }}
        ></div>
        <div
          className="absolute top-1/3 right-1/4 w-20 h-20 border border-white/15 rounded-lg rotate-45 animate-pulse"
          style={{ animationDelay: "1.5s" }}
        ></div>
      </div>

      <div className="relative z-10 min-h-screen flex flex-col justify-between items-center p-4 max-w-4xl mx-auto gap-10">
        <div className="w-full h-full items-center flex flex-col mt-[30vh] md:m-auto gap-3">
          <div className="w-full max-w-2xl flex flex-col justify-center">

            <form onSubmit={generateShortLink}>
              <div className="flex flex-col">
                <div className="md:hidden w-full space-y-2">
                  <div className="flex flex-row border border-gray-700 rounded-lg bg-gray-900/50">
                    <div className="w-30 border-r border-gray-700 text-center m-auto">
                      <Select value={expiration} onValueChange={setExpiration}>
                        <SelectTrigger className="h-12 bg-transparent border-0 text-white rounded-none focus:ring-0 focus:ring-offset-0 focus-visible:ring-0 m-auto cursor-pointer">
                          <div className="flex items-center justify-center w-full">
                            <SelectValue />
                          </div>
                        </SelectTrigger>
                        <SelectContent>
                          {expirationOptions.map((option) => (
                            <SelectItem key={option.value} value={option.value} className="cursor-pointer">
                              {option.label}
                            </SelectItem>
                          ))}
                        </SelectContent>
                      </Select>
                    </div>
                    <div className="flex-1">
                      <Input
                        type="url"
                        placeholder="Enter your URL here..."
                        value={url}
                        onChange={(e) => setUrl(e.target.value)}
                        required
                        className="h-12 text-sm md:text-lg bg-transparent border-0 text-white placeholder:text-gray-400 focus:ring-0 focus:ring-offset-0 focus-visible:ring-0 rounded-none"
                      />
                    </div>
                  </div>

                  <div className="w-full">
                    <Button
                      type="submit"
                      className="h-12 text-sm bg-white text-black hover:bg-gray-100 font-mono tracking-wider w-full cursor-pointer rounded-lg"
                      disabled={isLoading}
                    >
                      {isLoading ? "SHORTENING..." : "SHORTEN"}
                    </Button>
                  </div>
                </div>

                {/* Desktop layout: horizontal row (unchanged) */}
                <div className="hidden md:flex w-full bg-gray-900/50 border border-gray-700 rounded-lg overflow-hidden flex-row">
                  {/* Expiration selector - styled to blend seamlessly */}
                  <div className="w-30 border-r border-gray-700 text-center m-auto">
                    <Select value={expiration} onValueChange={setExpiration}>
                      <SelectTrigger className="h-12 bg-transparent border-0 text-white rounded-none focus:ring-0 focus:ring-offset-0 focus-visible:ring-0 m-auto cursor-pointer">
                        <div className="flex items-center justify-center w-full">
                          <SelectValue />
                        </div>
                      </SelectTrigger>
                      <SelectContent>
                        {expirationOptions.map((option) => (
                          <SelectItem key={option.value} value={option.value} className="cursor-pointer">
                            {option.label}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>

                  {/* URL input - styled to blend seamlessly */}
                  <div className="flex-1">
                    <Input
                      type="url"
                      placeholder="Enter your URL here..."
                      value={url}
                      onChange={(e) => setUrl(e.target.value)}
                      required
                      className="h-12 text-lg bg-transparent border-0 text-white placeholder:text-gray-400 focus:ring-0 focus:ring-offset-0 focus-visible:ring-0 rounded-none"
                    />
                  </div>

                  {/* Generate button - styled to blend seamlessly */}
                  <div className="border-l border-gray-700">
                    <Button
                      type="submit"
                      className="h-12 text-sm bg-white text-black hover:bg-gray-100 font-mono tracking-wider w-32 cursor-pointer border-0 rounded-none"
                      disabled={isLoading}
                    >
                      {isLoading ? "SHORTENING..." : "SHORTEN"}
                    </Button>
                  </div>
                </div>
              </div>
            </form>
          </div>

        {history.length > 0 && (
          <div className="w-full max-w-2xl">
            <div className="flex items-center gap-2 mb-4 text-gray-400">
              <Clock className="w-4 h-4" />
              <span className="text-sm font-medium">Recent Links</span>
            </div>
            <div className="space-y-2">
              {history.map((item) => (
                <div
                  key={item.id}
                  className={`bg-gray-900/30 border border-gray-800 rounded-lg p-4 backdrop-blur-sm ${isExpired(item) ? "opacity-50" : ""}`}
                >
                  <div className="space-y-2">
                    <div className="flex items-center justify-between gap-4">
                      <div className="text-sm text-gray-400 truncate flex-1">{item.originalUrl}</div>
                      <div className="text-xs text-gray-500 flex flex-col items-end">
                        <div>{new Date(item.createdAt).toLocaleDateString()}</div>
                        <div className={`text-xs ${isExpired(item) ? "text-red-400" : "text-gray-500"}`}>
                          {item.expiration === "permanent"
                            ? "Permanent"
                            : isExpired(item)
                              ? "Expired"
                              : formatTimeRemaining(item.expiresAt!)}
                        </div>
                      </div>
                    </div>
                    <div className="flex items-center justify-between gap-2 flex flex-row">
                      <code
                        className={`text-sm font-mono ${isExpired(item) ? "text-gray-500 line-through" : "text-white"}`}
                      >
                        {item.shortUrl}
                      </code>
                      <div className="flex items-center gap-1">
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => copyToClipboard(item.shortUrl)}
                          className="h-6 w-6 p-0 text-gray-400 hover:text-white hover:bg-transparent cursor-pointer"
                          disabled={isExpired(item)}
                        >
                          <Copy className="w-3 h-3" />
                        </Button>
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => showLinkPopup(item.shortUrl)}
                          className="h-6 w-6 p-0 text-gray-400 hover:text-white hover:bg-transparent cursor-pointer"
                          disabled={isExpired(item)}
                        >
                          <ExternalLink className="w-3 h-3" />
                        </Button>
                      </div>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}
        </div>
        
        <Footer />
        
      </div>

      <Dialog open={showPopup} onOpenChange={setShowPopup}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <QrCode className="w-5 h-5" />
              Your Short Link is Ready!
            </DialogTitle>
            <DialogDescription>Share your shortened URL or scan the QR code</DialogDescription>
          </DialogHeader>
          <div className="space-y-4">
            <div className="flex justify-center">
              <div className="bg-white p-4 rounded-lg border">
                <QRCodeSVG value={shortLink} className="w-48 h-48" />
              </div>
            </div>

            <div className="space-y-2">
              <Label>Short Link</Label>
              <div className="flex gap-2">
                <Input
                  value={shortLink}
                  readOnly
                  className="font-mono text-sm"
                  onClick={(e) => e.currentTarget.select()}
                />
                <Button type="button" variant="outline" size="icon" onClick={() => copyToClipboard(shortLink)}>
                  <Copy className="w-4 h-4" />
                </Button>
              </div>
            </div>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  )
}
