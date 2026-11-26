# Repository instructions

Start with the README. Read the system design before changing the request or SSE contract. Read the operations guide before running a comparison.

Preserve the distinction between first visible text, reported tokens, and stream events. Retain failed trials and partial responses. Keep the fixture independent of model servers.

Run `make check` after each behavioral change. Exercise the installed CLI outside the checkout after a packaging change. Update the diagrams and regenerate the source map when a change affects them.

Use `graphify query "run_trial"` to locate related code. Run `make graph` after source structure changes.
