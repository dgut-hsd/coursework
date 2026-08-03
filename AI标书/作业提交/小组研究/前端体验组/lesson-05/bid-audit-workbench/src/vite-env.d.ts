/// <reference types="vite/client" />

declare module 'eventsource' {
  export type EventSourceReadyState = 0 | 1 | 2;
  export interface MessageEvent {
    data: string;
    type: string;
    origin: string;
    lastEventId: string;
  }
  export interface EventSourceInit {
    withCredentials?: boolean;
    headers?: Record<string, string>;
  }
  export class EventSource {
    constructor(url: string, eventSourceInitDict?: EventSourceInit);
    onopen: ((this: EventSource, ev: Event) => unknown) | null;
    onmessage: ((this: EventSource, ev: MessageEvent) => unknown) | null;
    onerror: ((this: EventSource, ev: Event) => unknown) | null;
    readyState: EventSourceReadyState;
    url: string;
    withCredentials: boolean;
    close(): void;
    addEventListener(
      type: string,
      listener: (this: EventSource, ev: unknown) => unknown,
    ): void;
    removeEventListener(
      type: string,
      listener: (this: EventSource, ev: unknown) => unknown,
    ): void;
    dispatchEvent(event: Event): boolean;
  }
}

declare module 'html-docx-js-typescript' {
  export function asBlob(
    html: string,
    options?: {
      orientation?: 'portrait' | 'landscape';
      margins?: { top?: number; right?: number; bottom?: number; left?: number };
    },
  ): Promise<Blob>;
}
