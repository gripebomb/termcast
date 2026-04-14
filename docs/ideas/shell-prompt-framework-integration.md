# Shell Prompt Framework Integration

## Status: Decided Not to Pursue (2026-04-13)

## Problem Statement

**How might we make TermCast's ambient weather display a first-class citizen in popular prompt frameworks — so users get seamless, performant weather in their prompt without manual shell scripting?**

## Why We Said No

After research, the value proposition doesn't justify the effort:

1. **No framework natively parses structured output.** Starship `[custom]`, P10k `p10k segment`, Oh My Posh `type: "command"`, and Tide custom items all display raw stdout as strings. Adding `--format json` to `--ambient` wouldn't unlock richer integrations — users would still see `☀️ 14°F` regardless of output format.

2. **Marginal improvement over existing `--install`.** TermCast already has `--install bash|zsh|tmux` that prints PS1/PROMPT_COMMAND snippets. The delta between "paste this in your `.bashrc`" and "paste this in your `starship.toml`" is small.

3. **Growth thesis is weak.** The bet was that framework-specific integrations would drive discovery via "starship weather" searches. In practice, you'd be competing for attention in framework docs/SEO for marginal payoff — users who want weather in their prompt already have `--install`.

4. **Maintenance burden is real.** Frameworks change config syntax. 3-4 framework snippets = 3-4 configs to keep current. This is documentation debt for a feature that doesn't differentiate.

5. **The cache is the real enabler, and it already exists.** The only technical concern was prompt render latency on cache miss, which the existing 15-min TTL cache already handles well.

## Variations Considered

| # | Variation | Verdict |
|---|-----------|---------|
| 1 | Documentation-only — just add framework snippets to README | Closest to viable but marginal over `--install` |
| 2 | `--format json` for `--ambient` | No framework parses it natively; power-user niche |
| 3 | `--install <framework>` smart installer | Over-engineered for the value delivered |
| 4 | Native Starship module (upstream PR) | Dependency on upstream maintainers; duplicate logic |
| 5 | Termcast as segment provider standard | Over-engineered; inventing a standard no one asked for |
| 6 | Combo: structured output + install snippets | Sounded good but JSON adds no value; snippets add marginal value |

## Framework Research Summary

| Framework | Custom command support | JSON parsing | Config mechanism |
|-----------|----------------------|-------------|-----------------|
| **Starship** | `[custom.xxx]` module with `command` prop | No — displays raw `$output` string | `starship.toml` |
| **Powerlevel10k** | Custom segment function via `p10k segment` | No — you format it in zsh | `.p10k.zsh` function |
| **Oh My Posh** | `type: "command"` segment with Go templates | Partial — `.Output` in templates | JSON/YAML/TOML config |
| **Tide** (fish) | Custom `_tide_item_xxx` fish function | No — you format it in fish | Fish universal variables |

## If We Revisit

Circumstances that could change the calculus:

- A major framework adds native weather or JSON segment parsing
- TermCast has enough user demand to justify the maintenance cost
- TermCast adds features that would benefit from structured output (e.g., multi-condition display, color-coded temperature ranges) that plain text can't express
- A framework maintainer reaches out requesting an integration
