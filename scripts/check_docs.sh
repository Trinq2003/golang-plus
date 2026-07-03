#!/usr/bin/env bash
# Keep the docs honest: README must defer to ROADMAP.md for per-feature status
# instead of restating it (which is how "complete" once ended up contradicting
# "Planned"). Run from the repo root.
set -euo pipefail
fail=0
note() { echo "docs-consistency: $*"; }

grep -qi "single source of truth" ROADMAP.md \
  || { note "ROADMAP.md must declare itself the single source of truth"; fail=1; }

grep -q "ROADMAP.md" README.md \
  || { note "README.md must link to ROADMAP.md"; fail=1; }

grep -qi "single source of truth" README.md \
  || { note "README.md must defer to ROADMAP.md as the single source of truth"; fail=1; }

# The blanket status claims we deliberately removed must not creep back in.
if grep -Eni 'v2 tooling is .{0,20}complete|v3 .{0,40}is .{0,20}complete' README.md; then
  note "README.md restated a blanket 'complete' status; keep per-feature status in ROADMAP.md"
  fail=1
fi

if [ "$fail" -eq 0 ]; then
  note "OK"
fi
exit "$fail"
