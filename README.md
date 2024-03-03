# github-lsp

`github-lsp` is an implementation of the [Language Server Protocol](https://microsoft.github.io/language-server-protocol/) for working with [GitHub Markdown](https://docs.github.com/en/get-started/writing-on-github/getting-started-with-writing-and-formatting-on-github/basic-writing-and-formatting-syntax) files.

This is a tool for getting link suggestions while writing READMEs and GitHub Wiki pages locally.

Use this LSP in conjunction with some other Markdown LSP if you want gotoDefinition et.al. This LSP only focuses on adding autocomplete to

- [x] `#` Issues and PRs
- [ ] `[` Wiki Pages
- [ ] `:` Organizations / Owners
- [ ] `/` Repositories
- [ ] `@` Users

[Issues](https://github.com/AlexanderBrevig/github-lsp/issues) and [PRs](https://github.com/AlexanderBrevig/github-lsp/pulls) are very welcome!

## Install

```shell
git clone git@github.com:alexanderbrevig/github-lsp # here you can see why : is for owners and / is for repositories
cd github-lsp
cargo install --path .
```

You can now configure your editor to use this LSP of STDIO.
