/* tslint:disable */
/* eslint-disable */

/* auto-generated by NAPI-RS */

export interface RequestOptions {
  headers?: Record<string, string>
  timeout?: number
}
export declare function fetchUrl(url: string, opts?: RequestOptions | undefined | null): Promise<Response>
export declare class Response {
  code: string
  body?: string
  constructor(code: string, body?: string)
}
