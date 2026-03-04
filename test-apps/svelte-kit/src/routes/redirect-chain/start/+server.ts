import { redirect } from '@sveltejs/kit'

export const GET = () => {
  throw redirect(302, '/redirect-chain/step-1')
}
