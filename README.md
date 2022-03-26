## cloudmirror

基于 Cloudflare Workers 实现的简单网页代理。

### 用法

访问 `https://cloudmirror.darkyzhou.workers.dev/${需要代理的网址名称}` 即可。

例如：

- GitHub：[https://cloudmirror.darkyzhou.workers.dev/https://github.com](https://cloudmirror.darkyzhou.workers.dev/https://github.com)
- NPM：[https://cloudmirror.darkyzhou.workers.dev/https://npmjs.com](https://cloudmirror.darkyzhou.workers.dev/https://npmjs.com)
- docs.rs：[https://cloudmirror.darkyzhou.workers.dev/https://docs.rs](https://cloudmirror.darkyzhou.workers.dev/https://docs.rs)

### 限制

- 许多基于 SPA 路由的网站无法正常实现页面跳转，例如百度。
- 许多网站限制了通过网页代理进行访问，例如谷歌全系网站。
- 有些网站的请求 url 是写死在 js 中的，难以修改。
- 登录几乎不能使用，也不应该使用。一方面，许多网站会检测当前是否通过网页代理访问进而阻止登录。另一方面，cloudmirror 的实现原理决定了即使登录多个网站，cookie 都会挂载到同一个域名（也就是 cloudmirror 的域名）下，并不算安全。

### 原理

根据输入的 url 发出请求，检查响应的 `content-type`。

1. 若 `content-type` 不是 `text/html`，直接返回给用户。
2. 否则，将 HTML 文本中的 `<a>`、`<link>`、`<style>` 等元素的相关 url 替换为经过 cloudmirror 代理的 url。具体见 `rewriter.rs`。

#### 边界情况

- 某些网站的搜索框会让用户跳转到像 `https://cloudmirror.darkyzhou.workers.dev/search?q=text` 这样的 url 上，此时只需根据请求的 `referer` 填充正确的 url 即可。
- （未实现）CSS 文件中也会通过 `url()` 的手段引用外部资源，需要重写。
