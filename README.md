# Walker

A tool to recursively check for broken links in a website. To make sure that it does check for all valid URls in the website, every single /:page would require a reference(a link) to another page. To avoid duplicate requests to the same link, we use hashmaps to store links of webpages that have already been verified to be have broken/working.