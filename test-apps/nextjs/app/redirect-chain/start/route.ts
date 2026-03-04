import { NextResponse } from 'next/server'

export function GET(request: Request) {
  return NextResponse.redirect(
    new URL('/redirect-chain/step-1', request.url),
    302
  )
}
