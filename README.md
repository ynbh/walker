# Walker

Walker is a tool that performs a recursive analysis of a website, including all subdirectories and pages, to search for faulty links. It operates under the premise that the homepage of the site in question leads to other pages within the website, which subsequently lead to yet more pages. To optimize speed, all URLs that have already been encountered are cached in a `HashSet` to prevent duplicate `fetch` requests. The `urls` `HashSet` contained within the `URLs` struct stores all URLs that return a successful `200 OK` response. All requests occur concurrently, so the process is fast as fuck.

It is worth noting that this process may take an indeterminate amount of time, as a website may contain an infinite number of nested links. A potential future update to the recursive function could be the addition of a `depth` parameter, which would restrict recursion to only a specified number of levels. I don't plan on implementing this any time soon, though!

## Quirks

### Client side rendering

Since the implementation of this tool works through fetching the HTML of the website in question, it would be impossible to detect, or perhaps even retrieve the initial HTML, for websites that render on the client.

Therefore, `walker` only works for static and server-rendered sites. It should be noted that when I mention server-rendered, I mean websites that fetch all HTML in their initial request to the server, and not just selective data like `meta` tags for bots to crawl.

There could be cases where some parts of the website are server-rendered, while some of them are client-side rendered. In these cases, `walker` will only parse and verify links it can find on the server-side rendered HTML.

I could perhaps use something like headless chrome to get the HTML for these pages, but that only adds overhead to problem I was initially trying to solve. I am not keen on making headless chrome work in Rust. That task sounds like a better job for TypeScript.

### URLs

URLs are shitty. Essentially, I eventually want to be able to discern between URLs like: https://example.com#id and https://example.com/#id. In the former case, https://example.com has already been verified to be have not broken, and hence, verifying https://example.com#id would be redundant, since it is not really a new URL. It's just a section within the page.

Also, for some reason, there are some duplicate requests happening even after I've made sure to memoize the URLs in a `HashSet`. I am not sure _why_ that happens, but it feels like a rust-specific quirk, since the same script works perfectly, albeit slow, in TypeScript. For example:

![Multiple requests](https://media.discordapp.net/attachments/841704583364608051/1063072047527366786/image.png)

There is a massive performance optimization here. But I quite don't know how to achieve that. Welp, I guess I will eventually figure it out.

### Rate limits

Since I don't wait between each request, some websites might enforce their rate-limiting policies on `walker`, and hence cause it to error out for URLs which are working perfectly fine. In these cases, if the API is returning semantic error codes, `walker` will display something like `429 Too Many Requests`.

`walker` has a timeout of 5 seconds between each request. If the URL does not return a response within 5 seconds, it will error out and show that the operation was timed out.

## Examples

Examples of what the data would look like after the process is over can be found in the [data](/data/) directory. All links in it are unique. However, the data simply only shows what links are _available_ on the website, and not if they are valid or not. To see what links are functional, download the binary from [download](/download/) and run it with a `--url` argument.

## Usage

Using `walker` is easy. It has a dead-simple CLI interface that can be used to visually see the results of the analysis. Options for it are:

```bash
Options:
  -u, --url <URL>  URL of the website to analyze links from
  -r, --relative   Whether to perform a deep search or not
  -d, --debug      Shows what URL walker is currently on
  -c, --construct  Constructs the stream of responses into a string and copies it to the clipboard
  -h, --help       Print help information
  -V, --version    Print version information
```

So, for example, doing:

```bash
walker --url "https://ynb.sh"
```

...would result in:

```bash
Received 4 links. Iterating now...
https://ynb.sh/posts: 200 OK
https://github.com/ynbh: 200 OK
https://ynb.sh: 200 OK
Stats
Time to get all links: 0 seconds
Time to verify links: 0 seconds
```

But when used with the `-r` argument, it would result in something like:

```bash
Received 17 links. Iterating now...
https://en.wikipedia.org/wiki/Test_of_English_as_a_Foreign_Language: 200 OK
https://ynb.sh/assets/toefl-listening.png: 200 OK
https://ynb.sh/assets/toefl-reading.png: 200 OK
https://ynb.sh/posts/preparing-for-and-writing-the-TOEFL: 200 OK
An error occurred: error sending request for url (http://www.icsscsummerofcode.com/): error trying to connect: dns error: failed to lookup address information: nodename nor servname provided, or not known
https://ynb.sh/posts: 200 OK
https://ynb.sh/assets/toefl-writing-template.png: 200 OK
https://www.toeflresources.com/speaking-section/toefl-speaking-templates: 200 OK
https://ynb.sh/assets/toefl-speaking.png: 200 OK
https://ynb.sh: 200 OK
https://ynb.sh/assets/toefl-writing.png: 200 OK
http://zeroclipboard.org: 500 Internal Server Error
https://github.com/ynbh: 200 OK
https://ynb.sh/posts/free-speech-and-some-concerns: 200 OK
https://ynb.sh/posts/black-panther-wakanda-forever-review: 200 OK
https://ynb.sh/assets/toefl-speaking-template.png: 200 OK
Stats
Time to get all links: 1 seconds
Time to verify links: 0 seconds
```

To debug what URL `walker` is currently fetching, simply pass it a `-d` debug flag.

In situations where `walker` is unable to resolve some arbitrary URL, it will properly show the error while verifying all the links. Often, this error occurs because `reqwest` is unable to resolve its DNS. You can check if that is the case by running `walker --url <URL> -s`. If it does not return an error, there is probably something else going on with the URL that needs to be looked at.

## Read

[Blog post detailing how I made this tool](https://ynb.sh/posts/walker)
