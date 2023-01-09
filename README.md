# Walker

Walker is a tool that performs a recursive analysis of a website, including all subdirectories and pages, to search for faulty links. It operates under the premise that the homepage of the site in question leads to other pages within the website, which subsequently lead to yet more pages. To optimize speed, all URLs that have already been encountered are cached in a `HashSet` to prevent duplicate `fetch` requests. The `urls` `HashSet` contained within the `URls` struct stores all URLs that return a successful `200 OK` response. 

It is worth noting that this process may take an indeterminate amount of time, as a website may contain an infinite number of nested links. A potential future update to the recursive function could be the addition of a `depth` parameter, which would restrict recursion to only a specified number of levels. I don't plan on implementing this any time soon, though!

## Quirks

Since the implementation of this tool works through fetching the HTML of the website in question, it would be impossible to detect, or perhaps even retrieve the initial HTML, for websites that render on the client. Therefore, `walker` only works for static and server-rendered sites. It should be noted that when I mention server-rendered, I mean websites that fetch all HTML in their initial request to the server, and not just selective data like `meta` tags for bots to crawl.

There could be cases where some parts of the website are server-rendered, while some of them are client-side rendered. In these cases, `walker` will only parse and verify links it can find on the server-side rendered HTML.

I could perhaps use something like headless chrome to get the HTML for these pages, but that only adds overhead to problem I was initially trying to solve. I am not keen on making headless chrome work in Rust. That task sounds like a better job for TypeScript.

## Examples

Examples of what the data would look like after the process is over can be found in the [data](/data/) directory. All links in it are unique. However, the data simply only shows what links are _available_ on the website, and not if they are valid or not. To see what links are functional, download the binary and run it with a `--url` argument.

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
https://github.com/ynbh: ✅
https://ynb.sh/posts: ✅
https://ynb.sh: ✅
```

But when used with the `-r` argument, it would result in something like:

```bash
Received 16 links. Iterating now...
http://zeroclipboard.org: ❌
https://ynb.sh/assets/toefl-speaking-template.png: ✅
https://ynb.sh/posts/free-speech-and-some-concerns: ✅
https://ynb.sh: ✅
https://ynb.sh/posts: ✅
https://github.com/ynbh: ✅
https://en.wikipedia.org/wiki/Test_of_English_as_a_Foreign_Language: ✅
https://ynb.sh/assets/toefl-speaking.png: ✅
https://ynb.sh/assets/toefl-writing-template.png: ✅
https://www.toeflresources.com/speaking-section/toefl-speaking-templates: ✅
https://ynb.sh/assets/toefl-listening.png: ✅
https://ynb.sh/assets/toefl-writing.png: ✅
https://ynb.sh/posts/black-panther-wakanda-forever-review: ✅
https://ynb.sh/posts/preparing-for-and-writing-the-TOEFL: ✅
https://ynb.sh/assets/toefl-reading.png: ✅
Stats
Time to get all links: 3 seconds
Time to verify links: 5 seconds
```

To debug what URL `walker` is currently fetching, simply pass it a `-d` debug flag. 

In situations where `walker` is unable to resolve some arbitrary URL, it will show a message stating `CANNOT RESOLVE ❌`. The URL is either no longer valid, or there is some kind of block that is making it unable to properly resolve the URL. Either way, it is best to visit the website itself and check if it loads for you.