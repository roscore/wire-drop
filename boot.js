// Same-origin loader (the CSP forbids inline scripts).
import init from './wd_web.js';

init({ module_or_path: new URL('wd_web_bg.wasm', import.meta.url) });
