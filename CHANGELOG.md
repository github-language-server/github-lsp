# 24.03.10

🎁 Stability and `hover`

After some feedback it was clear that the incremental text handling is _hard_ to get right across editors.

This version uses full document sync (much like most LSPs I've checked).

There's also a timeout configured for a few async operations that could hang the UX before.

feat: initial hover functionality (#46)
fix: remove email from author label (#45)
fix: bug with : and invalid searches (#42)

# 24.03.08

🚀 Our first release!

✨ We already have a contributor; thank you krfl!

I solemnly swear to make a cleaner and better CHANGELOG for the next release,
but since this is our first. Here it is, a shortened list of interesting commits that got us here.

ci: add release workflow (#39)
fix: title for issues (#38)
fix: improve gh errors (#35)
fix: much more stable, no longer panic based on out of bounds (#33)
fix: improve issue status readavility, close #24 (#31) <Kristoffer Flottorp>
fix(issues): list all statuses (#29)
fix: wiki render absolute links, close #26 (#27)
docs: move to org [skip ci] (#21)
docs: add asciinema demo (#20)
