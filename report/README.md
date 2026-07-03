# GoPlus — technical report

`main.tex` is a formal, self-contained LaTeX report about GoPlus (HUST red/white
theme). It compiles with **pdflatex** only — no external tools required.

## Before you build

1. **Fill the placeholders** on the title page of `main.tex` — the fields wrapped
   in `⟨ … ⟩`: author, student ID, programme, supervisor, course, date, and the
   school/faculty line.
2. **(Optional) Add the logo**: drop a `logo.png` next to `main.tex` and it is
   shown automatically; otherwise a placeholder box appears.
3. **(Optional) Brand red**: adjust `\definecolor{hustred}{HTML}{A31621}` near the
   top to your exact HUST brand hex.

## Build

```bash
pdflatex main.tex     # run twice so the table of contents resolves
pdflatex main.tex
```

Or upload `main.tex` (and `logo.png`) to Overleaf and compile there.

The report is intentionally high-level: it explains the concepts and shows small
illustrative code, without referencing the compiler's internal implementation.
