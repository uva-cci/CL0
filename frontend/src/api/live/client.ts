// Placeholder client until codegen exists for your .proto files.
// Swap this with the generated service client from @connectrpc when ready.
import { createPromiseClient } from '@connectrpc/connect'
import { liveTransport } from './transport'

export interface LiveService {
  streamEvents(req: { since?: number }): AsyncIterable<{ event: unknown }>
}

// Note: createPromiseClient normally takes a generated service descriptor.
// Here we cast to keep the scaffold compiling; replace with real descriptor later.
export const liveClient = createPromiseClient<any>({} as any, liveTransport)