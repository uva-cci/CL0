export type LiveEvent = {
  id: string
  ts: number
  channel: 'repl' | 'graph' | 'values'
  payload: unknown
}