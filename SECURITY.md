# Security

## Reporting a problem

Please do not open a public issue for a security problem. Report it privately instead, through
GitHub's [private vulnerability reporting](https://github.com/Srynetix/pastille/security/advisories/new)
(the *Report a vulnerability* button under the repository's *Security* tab).

Say what the problem is, how to reproduce it (a circuit file, if one triggers it), and what
someone could do with it. You will get an answer within a week. Once it is fixed and released,
you are credited in the advisory unless you prefer not to be.

## What is supported

Only the latest release, and the web app as GitHub Pages serves it: fixes go into a new
release, not into old ones.

## What Pastille does with your data

Pastille has no server and no account. The web app runs entirely in the browser and keeps
circuits in its local storage; the desktop app reads and writes only the files you open and
save. Neither sends your circuits anywhere: once loaded, the only thing that leaves is a
datasheet link, opened in your browser when you click it, and the desktop app follows only
`https://` links. A way for a circuit file, an example or a program to
make Pastille do otherwise is the kind of problem worth reporting.

The desktop apps are not signed: macOS and Windows warn about them the first time. Download
them only from this repository's releases, and check them against the `.sha256` files
published beside them.
