## cloud-mirror

基于 Cloudflare Workers 实现的简单网页代理。不仅可以用来下载像 GitHub 的文件，也支持在像 GitHub 这样的网站进行基本的导航。

### 用法

访问 `https://p.zqy.io/网址` 即可。

> 也可使用 `http`，适用于一些没有安装 https 证书的环境，安全风险自负。

例如：

- GitHub：[https://p.zqy.io/https://github.com](https://p.zqy.io/https://github.com)
- NPM：[https://p.zqy.io/https://npmjs.com](https://p.zqy.io/https://npmjs.com)
- Rust Playground：[https://p.zqy.io/https://www.rust-lang.org/](https://p.zqy.io/https://www.rust-lang.org/)（进去后点上面的“Playground”）
- 下载一个 GitHub 的文件：[https://p.zqy.io/https://github.com/darkyzhou/blog-house/archive/refs/tags/0.3.0.zip](https://p.zqy.io/https://github.com/darkyzhou/blog-house/archive/refs/tags/0.3.0.zip)

### 限制

- 暂不支持同时打开多个**不同**的网站（打开多个相同网站的标签页是可以的），原因请见下面的原理。这更多地是一种设计上的取舍。
- 许多网站限制了通过网页代理进行访问，例如谷歌全系网站。
- 有些网站的请求 url 是写死在 js 中的，难以修改。
- 登录几乎不能使用，也不应该使用。一方面，许多网站会检测当前是否通过网页代理访问进而阻止登录。另一方面，cloud-mirror 的实现原理决定了即使登录多个网站，cookie 都会挂载到同一个域名（也就是 cloud-mirror 的域名）下，并不算安全。

### 原理

根据输入的 url：

- 若 url 的 path 不为 `/`，我们假定用户这时在下载一些文件，例如 `.js`、`.zip` 等，这时 cloud-mirror 直接发出请求，并将响应传给浏览器。
- 若 url 的 path 为 `/`，例如访问 `http://p.zqy.io/https://github.com` 时，cloud-mirror 通过设置 cookie 让浏览器在之后的请求中能够告诉 cloud-mirror 目前正在访问哪个网站，然后返回一个 302，指向 `http://p.zqy.io`。请求发出后，检查响应的 `content-type`：
  - 若 `content-type` 不是 `text/html`，直接返回给浏览器。
  - 否则，将 HTML 文本中的 `<a>`、`<link>`、`<style>` 等元素的相关 url 替换为经过 cloud-mirror 代理的 url。具体见 `rewriter.rs`。

#### 边界情况

- 有些资源可能会出现 CORS 异常，此时手动添加一个 `Access-Control-Allow-Origin` 头即可。
- （未实现）CSS 文件中也会通过 `url()` 的手段引用外部资源，需要重写。
- （未实现）一些国际性的 CDN 的 url 可以通过白名单过滤掉，例如 jsdelivr。

### 本地开发

1. 安装并配置好 [wrangler](https://github.com/cloudflare/wrangler)。
2. 运行 `wrangler dev -e dev`。

### 部署

> 你需要了解 Cloudflare Workers 的相关概念，请通过搜索引擎查找相关资料。如果不想这么麻烦，直接用作者的 `p.zqy.io` 即可。

1. 修改 `wrangler.toml` 中的 `vars.DOMAIN`。
2. 运行 `wrangler publish`。
