# github-lsp

[![https://asciinema.org/a/645196](demo.gif)](https://asciinema.org/a/645196)

[![CI](https://github.com/github-language-server/github-lsp/actions/workflows/ci.yml/badge.svg)](https://github.com/github-language-server/github-lsp/actions/workflows/ci.yml)

https://asciinema.org/a/645195

`github-lsp` is an implementation of the [Language Server Protocol](https://microsoft.github.io/language-server-protocol/) for working with [GitHub Markdown](https://docs.github.com/en/get-started/writing-on-github/getting-started-with-writing-and-formatting-on-github/basic-writing-and-formatting-syntax) files.

This is a tool for getting link suggestions while writing READMEs and GitHub Wiki pages locally.

Use this LSP in conjunction with some other Markdown LSP if you want gotoDefinition et.al. This LSP only focuses on adding autocomplete to

* [x] `#` Issues and PRs
* [x] `[` Public Wiki Pages
* [x] `:` Organizations / Owners
* [x] `/` Repositories (yours and the orgs you are part of, no global search yet)
* [x] `@` Organization Members

[Issues](https://github.com/github-language-server/github-lsp/issues) and [PRs](https://github.com/github-language-server/github-lsp/pulls) are very welcome!

## Requirements

This LSP uses the amazing [gh](https://cli.github.com/) so you will need to install that and auth with it.
We currently use it for retrieving your auth token, and for meta about the current repo.

```shell
gh auth login
```

## Install

```shell
cargo install --git https://github.com/github-language-server/github-lsp
```

```shell
git clone git@github.com:alexanderbrevig/github-lsp # here you can see why : is for owners and / is for repositories
cd github-lsp
cargo install --path .
```

You can now configure your editor to use this LSP on `stdio`.

## Items added by LSP

### `#` trigger

[#1](https://github.com/github-language-server/github-lsp/issues/1)
[#2](https://github.com/github-language-server/github-lsp/issues/2)

### `@` trigger

[AlexanderBrevig](https://github.com/AlexanderBrevig)

### `:` trigger

[github-language-server](https://github.com/github-language-server)

### `/` trigger

[github-language-server/github-lsp](https://github.com/github-language-server/github-lsp)

### `[` trigger (Home is always suggested)

[Home](https://github.com/github-language-server/github-lsp/wiki)
