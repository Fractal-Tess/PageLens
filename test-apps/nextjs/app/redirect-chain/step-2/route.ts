import { NextResponse } from 'next/server'

export function GET(request: Request) {
  return NextResponse.redirect(
    new URL('/redirect-chain/final', request.url),
    302
  )
}
