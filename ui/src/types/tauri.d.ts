export {};

declare global {
  interface Window {
    __TAURI__: {
      core: {
        invoke(cmd: string, args?: Record<string, unknown>): Promise<any>;
      };
      event: {
        listen(event: string, cb: (e: { payload: any }) => void): Promise<() => void>;
      };
    };
  }
}
