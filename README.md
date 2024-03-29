# Walker

Walker is a tool that performs a recursive analysis of a website to search for faulty links. It operates under the premise that the homepage of the site in question leads to other pages within the website, which subsequently lead to yet more pages.

It is worth noting that this process may take an indeterminate amount of time, as a website may contain an infinite number of nested links. A potential future update to the recursive function could be the addition of a `depth` parameter, which would restrict recursion to only a specified number of levels. I don't plan on implementing this any time soon due to CBSE board exams and college applications, though!

Also, `walker` is fast af. After it acquires all the URLs in a website, it sends them parallel requests to quickly verify if they're broken or not. There is a [rate-limiting issue](#rate-limits) that I am quite not sure how to fix, but I think I'll eventually figure it out. Foe what it's worth, the only cost-heavy part of `walker` is acquiring the URLs. That process is neither concurrent, nor parallel, and hence takes a while.

A diagrammatic representation of the process:
![Walker Rep](./Walker-2.jpg)

## Quirks

### Client side rendering

Since the implementation of this tool works through fetching the HTML of the website in question, it would be impossible for it to retrieve HTML for pages that render on the client. Basically, only the HTML available when you view the page source is analyzed. This often includes websites that are either statically generated or rendered on the server.

### Rate limits

Since I don't wait between each request, some websites might enforce their rate-limiting policies on `walker`, and hence cause it to error out for URLs which are working perfectly fine. In these cases, if the API is returning semantic error codes, `walker` will display something like `429 Too Many Requests`.

`walker` has a timeout of 5 seconds between each request. If the URL does not return a response within 5 seconds, it will error out and show that the operation was timed out.

### Head requests

To save on bytes, `walker` performs `HEAD` requests instead of `GET` requests. However, some websites might deny responding to this method, which could lead to false negatives. In these cases, `walker` will show an error like `405 Method Not Allowed`.


## Install

To install `walker`, you will need to have `cargo` configured in your environment. You can see how to do so [here](https://doc.rust-lang.org/cargo/getting-started/installation.html). When you're done doing so, run:
```zsh
cargo install link-walker
```
...and let it install. Once `link-walker` is done installing, you can simply run:
```zsh
link-walker -h
```
...to get information on how to use the tool.

## Usage

Using `walker` is easy. It has a dead-simple CLI interface that can be used to visually see the results of the analysis. Options for it are:

```bash
Usage: link-walker [OPTIONS] --url <URL>

Options:
  -u, --url <URL>  URL of the website to analyze links from
  -r, --relative   Whether to perform a deep search or not
  -d, --debug      Shows what URL walker is currently on
  -c, --construct  Constructs the stream of responses into a string and copies it to the clipboard
  -s, --singular   Checks if the domain of the URL is resolvable
  -h, --help       Print help
  -V, --version    Print version
```

So, for example, doing:

```bash
link-walker --url "https://ynb.sh"
```

...would result in:

```bash
Received 3 links. Iterating now...
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

In situations where `walker` is unable to resolve some arbitrary URL, it will accurately display the exact error that occurred. Often, this error occurs because `reqwest` is unable to resolve its DNS. You can check if that is the case by running `link-walker --url <URL> -s`. If it does not return an error, there is probably something else going on with the URL that needs to be looked at.

## Extra

Note that this is an experimental tool, and I only made this to learn more about Rust. I wrote a [blog post detailing how I made it!](https://ynb.sh/posts/walker)
