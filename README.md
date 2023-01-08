# Walker

Walker is a tool that performs a recursive analysis of a website, including all subdirectories and pages, to search for faulty links. It operates under the premise that the homepage of the site in question leads to other pages within the website, which subsequently lead to yet more pages. To optimize speed, all URLs that have already been encountered are cached in a `HashSet` to prevent duplicate `fetch` requests. The `urls` vector contained within the `URls` struct stores all URLs that return a successful `200 OK` response. 

It is worth noting that this process may take an indeterminate amount of time, as a website may contain an infinite number of nested links. A potential future update to the recursive function could be the addition of a `depth` parameter, which would restrict recursion to only a specified number of levels. I don't plan on implementing this any time soon, though!

## Quirks

Since the implementation of this tool works through fetching the HTML of the website in question, it would be impossible to detect, or perhaps even retrieve the initial HTML, for websites that render on the client. Therefore, `walker` only works for static and server-rendered sites. It should be noted that when I mention server-rendered, I mean websites that fetch all HTML in their initial request to the server, and not just selective data like `meta` tags for bots to crawl.

## Examples

Examples of what the data would look like after the process is over can be found in the [data](/data/) directory. All links in it are unique. However, the data simply only shows what links are _available_ on the website, and not if they are valid or not. Implementing that feature isn't hard, since we only need to check for `200 OK` responses, like I initially said in the introduction.
