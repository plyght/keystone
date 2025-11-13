'use client'

import { useEffect } from 'react'
import { useRouter } from 'next/navigation'
import { supabase } from '@/lib/supabase'

export default function AuthCallback() {
  const router = useRouter()

  useEffect(() => {
    const handleAuthCallback = async () => {
      try {
        const hashParams = new URLSearchParams(window.location.hash.substring(1))
        const accessToken = hashParams.get('access_token')
        const refreshToken = hashParams.get('refresh_token')
        const error = hashParams.get('error')
        const errorDescription = hashParams.get('error_description')

        if (error) {
          console.error('OAuth error:', error, errorDescription)
          router.push('/?error=' + encodeURIComponent(errorDescription || error))
          return
        }

        if (accessToken && refreshToken) {
          const { data, error: sessionError } = await supabase.auth.setSession({
            access_token: accessToken,
            refresh_token: refreshToken,
          })

          if (sessionError) throw sessionError

          if (data.session) {
            router.push('/')
            router.refresh()
          } else {
            router.push('/?error=auth_failed')
          }
        } else {
          const { data, error: sessionError } = await supabase.auth.getSession()
          if (sessionError) throw sessionError

          if (data.session) {
            router.push('/')
            router.refresh()
          } else {
            router.push('/?error=no_session')
          }
        }
      } catch (error: any) {
        console.error('Auth callback error:', error)
        router.push('/?error=' + encodeURIComponent(error.message || 'auth_failed'))
      }
    }

    handleAuthCallback()
  }, [router])

  return (
    <div className="min-h-screen flex items-center justify-center">
      <div className="text-center">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500 mx-auto mb-4"></div>
        <p className="text-gray-600">Completing sign in...</p>
      </div>
    </div>
  )
}

