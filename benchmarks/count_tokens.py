"""Count LLM tokens (and lines/chars) for the files given as arguments.

Uses the real GPT-4-family tokenizer (tiktoken cl100k_base) when available, so
the "an AI agent writes fewer tokens in GoPlus" claim is measured, not guessed.
Falls back to a labelled regex proxy if tiktoken cannot be installed.

    pip install tiktoken
    python count_tokens.py a.gp a.go ...
"""

import sys

try:
    import tiktoken

    enc = tiktoken.get_encoding("cl100k_base")

    def toks(s):
        return len(enc.encode(s))

    mode = "tiktoken cl100k_base (GPT-4 family)"
except Exception:
    import re

    def toks(s):
        return len(re.findall(r"\w+|[^\w\s]", s))

    mode = "PROXY regex word/punct split (NOT an official tokenizer)"

print(f"# tokenizer: {mode}")
print(f"{'file':40} {'lines':>7} {'chars':>8} {'tokens':>8}")
for path in sys.argv[1:]:
    with open(path, encoding="utf-8") as f:
        text = f.read()
    print(f"{path.split('/')[-1]:40} {text.count(chr(10)):>7} {len(text):>8} {toks(text):>8}")
