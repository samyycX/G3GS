"use client"

import { useEffect, useState } from "react"

interface FooterProps {
  className?: string
}

export default function Footer({ className = "" }: FooterProps) {
 
  return (
    <div 
      className={className}
    >
      <footer className="py-1 md:py-8 mt-auto">
        <div className="container mx-auto px-4">
          <div className="text-xs md:text-md text-center text-gray-300 flex flex-col gap-1">
            <p>&copy; 2025 https://g3.gs</p>
          </div>
        </div>
      </footer>
    </div>
  )
}