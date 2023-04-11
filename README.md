# reverse-proxy-rs

I like to learn thing by doing them.
This is a https reverse proxy impelmented in Rust with Axum and Hyper.

It is stateless, and the proxy works by conversing the URI path to a URL.

The example below has the proxy running on localhost:8080, and by calling `http://localhost:8080/httpbin.org/json`, I get reverse proxied to `https://httpbin.org/json`

```bash
$ curl http://localhost:8080/httpbin.org/json
➜
{
  "slideshow": {
    "author": "Yours Truly",
    "date": "date of publication",
    "slides": [
      {
        "title": "Wake up to WonderWidgets!",
        "type": "all"
      },
      {
        "items": [
          "Why <em>WonderWidgets</em> are great",
          "Who <em>buys</em> WonderWidgets"
        ],
        "title": "Overview",
        "type": "all"
      }
    ],
    "title": "Sample Slide Show"
  }
}
```
