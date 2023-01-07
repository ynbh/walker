# Walker

Walker is a tool that performs a recursive analysis of a website, including all subdirectories and pages, to search for faulty links. It operates under the premise that the homepage of the site in question leads to other pages within the website, which subsequently lead to yet more pages. To optimize speed, all URLs that have already been encountered are cached in a `HashSet` to prevent duplicate `fetch` requests. The `urls` vector contained within the `URls` struct stores all URLs that return a successful 200 OK response. 

It is worth noting that this process may take an indeterminate amount of time, as a website may contain an infinite number of nested links. A potential future update to the recursive function could be the addition of a `depth` parameter, which would restrict recursion to only a specified number of levels. I don't plan on implementing this any time soon, though!
